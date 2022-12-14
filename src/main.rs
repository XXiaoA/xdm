use anyhow::{Context, Result as aResult};
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use serde_yaml::{Mapping, Value};
use std::{fs, io::ErrorKind, path::Path};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// start xdm
    Start(Start),
    /// alias for `start`
    S(Start),
    /// link the specific dir\file
    Link(Link),
    /// alias for `link`
    L(Link),
    /// add link
    Add(Add),
    /// alias for `Add`
    A(Add),
}

#[derive(Args)]
struct Start {
    #[clap(default_value_t = String::from("xdm.yaml"), value_parser)]
    path: String,
    /// create all link including `manual`
    #[clap(short, long, value_parser)]
    all: bool,
}

#[derive(Args)]
struct Link {
    #[clap(value_parser)]
    path: String,
}

#[derive(Args)]
struct Add {
    #[clap(value_parser)]
    path: String,
}

trait Configuration {
    fn get_link_parameter(&self, original: &str, parameter: &str) -> &str;
}

impl Configuration for Value {
    fn get_link_parameter(&self, original: &str, parameter: &str) -> &str {
        let all_links = self.get("link").unwrap();
        let all_parameters = all_links.get(original).unwrap_or_else(|| -> &Value {
            if original.len() > 1 && &original[..2] == "./" {
                // ./a to a
                all_links.get(&original[2..]).unwrap()
            } else {
                all_links
                    .get(format!("./{}", original))
                    .unwrap_or_else(|| panic!("Can't find `{}` in configuration", original))
            }
        });
        if all_parameters.is_string() && parameter == "path" {
            all_parameters.as_str().unwrap()
        } else if let Some(value) = all_parameters.get(&parameter) {
            if value.is_bool() {
                match value {
                    Value::Bool(true) => "true",
                    Value::Bool(false) => "false",
                    _ => "",
                }
            } else {
                value.as_str().unwrap_or("")
            }
        } else {
            match parameter {
                "exist" | "if" | "create" | "relink" => "true",
                "force" | "manual" => "false",
                _ => "",
            }
        }
    }
}

fn get_conf() -> aResult<Value> {
    // get file path from user
    let command = Cli::parse().command;
    let file_path = if let Commands::S(args) | Commands::Start(args) = &command {
        &args.path
    } else {
        "xdm.yaml"
    };

    let file_content = std::fs::File::open(file_path).context("Failed to read configuration")?;
    serde_yaml::from_reader(file_content).context("Failed to read configuration")
}

fn main() {
    let xdm_config = get_conf();
    if let Err(err) = xdm_config {
        eprintln!("{err}");
        return;
    }
    let xdm_config = xdm_config.unwrap();

    let cli = Cli::parse();

    let add = if let Commands::A(args) | Commands::Add(args) = &cli.command {
        args.path.as_str()
    } else {
        ""
    };
    if !add.is_empty() {
        let link_path = fs::read_link(add);
        if link_path.is_err() {
            let err_kind = link_path.err().unwrap().kind();
            match err_kind {
                ErrorKind::NotFound => eprintln!("File or directory not exist"),
                ErrorKind::InvalidInput => eprintln!("This not a link"),
                _ => eprintln!("Something wrong"),
            }
            return;
        }
        let mut conf = xdm_config;
        let modified_conf = conf.get_mut("link").unwrap().as_mapping_mut().unwrap();
        modified_conf.insert(Value::String(absolute_path(add)), {
            let mut new_mapping = Mapping::new();
            let link_path = absolute_path(link_path.as_ref().unwrap().to_str().unwrap());
            new_mapping.insert(Value::String("path".to_owned()), Value::String(link_path));
            serde_yaml::Value::Mapping(new_mapping)
        });
        let config_path = if let Commands::S(args) | Commands::Start(args) = &cli.command {
            &args.path
        } else {
            "xdm.yaml"
        };
        let file = fs::File::create(config_path).unwrap();
        serde_yaml::to_writer(file, &conf).unwrap();
        let hint = format!(
            "add {:?} into configuration successfully",
            link_path.unwrap()
        )
        .green();
        println!("{hint}");
        return;
    }

    // create the specific link
    let path = if let Commands::L(args) | Commands::Link(args) = &cli.command {
        &args.path
    } else {
        ""
    };
    if !path.is_empty() {
        let link = xdm_config.get_link_parameter(path, "path");
        if !link.is_empty() {
            if let Err(err) = create_softlink(path, link) {
                println!("{}", format!("{}: {}", path, err).blue())
            } else {
                println!("{} {} {}", path.green(), "=>".green(), link.green());
            }
        }
        return;
    }

    // do jobs for create
    let all_creates = xdm_config.get("create");
    if let Some(..) = all_creates {
        println!("######create######");
        let all_creates = all_creates.unwrap().as_sequence().unwrap();

        for dir in all_creates {
            let dir = dir.as_str().unwrap();
            let path = absolute_path(dir);
            let path = Path::new(&path);
            if !path.exists() {
                fs::create_dir_all(path).unwrap();
                println!(
                    "{}{}",
                    dir.green(),
                    ": created the directory successfully".green()
                )
            }
        }
    }

    // do jobs for link
    let all_link_items = xdm_config.get("link");
    if all_link_items.is_none() {
        eprintln!("Can't find `link`");
        return;
    }
    println!("#######link#######");
    for link_item in all_link_items.unwrap().as_mapping().unwrap() {
        let original = link_item.0.as_str().unwrap();
        let link = xdm_config.get_link_parameter(original, "path");

        if link.is_empty() {
            println!(
                "{}{}",
                original.color("red"),
                ": something wrong in `path`".red()
            )
        } else {
            let manual: bool = xdm_config
                .get_link_parameter(original, "manual")
                .parse()
                .unwrap();
            let all = if let Commands::Start(args) | Commands::S(args) = &cli.command {
                args.all
            } else {
                false
            };
            if !manual || all {
                if let Err(err) = create_softlink(original, link) {
                    println!("{}", format!("{}: {}", original, err).blue())
                } else {
                    println!("{} {} {}", original.green(), "=>".green(), link.green());
                }
            }
        }
    }
}

/// Check a command is true or false WIP
///
/// * `command`: parameter `if`
fn get_command_status(command: &str) -> bool {
    use std::process::Command;

    let command: Vec<&str> = command.split_whitespace().collect();
    let status = Command::new(command[0])
        .args(&command[1..])
        .output()
        .unwrap()
        .status;

    status.success()
}

fn remove_file_dir(path: &Path) -> aResult<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.is_file() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn absolute_path(path: &str) -> String {
    use path_absolutize::Absolutize;
    use std::env;

    if &path[..1] == "/" {
        return path.to_string();
    }
    let path_vec: Vec<&str> = path.split('/').collect();

    if path_vec[0] == "~" {
        let prev = env::var("HOME").unwrap_or_else(|_| "none".to_string());
        let end = path_vec[1..].join("/");
        format!("{}/{}", prev, end)
    } else if &path_vec[0][..1] == "$" {
        let env = &path_vec[0][1..];
        let prev = env::var(env).unwrap_or_else(|_| "none".to_string());
        let end = path_vec[1..].join("/");
        format!("{}/{}", prev, end)
    } else {
        let absolute_path = Path::new(path).absolutize().unwrap();
        absolute_path.to_str().unwrap().to_string()
    }
}

/// auto create soft link with relative paths
///
/// * `original`: original path
/// * `link`: linked path
fn create_softlink(original: &str, link: &str) -> Result<(), String> {
    use std::os::unix::fs::symlink;

    fn can_create(original: &str, link: &str) -> Result<String, String> {
        let absolute_original = &absolute_path(original);
        let absolute_link = &absolute_path(link);

        let original_path = Path::new(absolute_original);
        let link_path = Path::new(absolute_link);

        let xdm_config = get_conf().unwrap();
        let exist: bool = xdm_config
            .get_link_parameter(original, "exist")
            .parse()
            .unwrap();
        let force: bool = xdm_config
            .get_link_parameter(original, "force")
            .parse()
            .unwrap();
        let relink: bool = xdm_config
            .get_link_parameter(original, "relink")
            .parse()
            .unwrap();
        let condition = xdm_config.get_link_parameter(original, "if");
        let command_status = if condition == "true" {
            true
        } else {
            get_command_status(condition)
        };

        if !command_status {
            Err(String::from("skip to create link"))
        } else if !exist && force {
            Ok(String::from("rc"))
        } else if !exist && !link_path.exists() {
            Ok(String::from("c"))
        } else if original_path.exists() && force {
            Ok(String::from("rc"))
        } else if !original_path.exists() {
            Err(format!("the file `{}` doesn't exist", original))
        } else if relink && link_path.is_symlink() {
            Ok(String::from("rc"))
        } else if !force && link_path.exists() {
            Err(format!(
                "`{}` has existed, you may need `force` parameter",
                link
            ))
        } else {
            Ok(String::from("c"))
        }
    }

    let can_create = can_create(original, link)?;

    let absolute_original = &absolute_path(original);
    let absolute_link = &absolute_path(link);
    let link_path = Path::new(absolute_link);
    let link_parent = link_path.parent().unwrap();

    let create: bool = get_conf()
        .unwrap()
        .get_link_parameter(original, "create")
        .parse()
        .unwrap();

    if create && !link_parent.exists() {
        fs::create_dir_all(link_parent.to_str().unwrap()).unwrap();
    }
    match can_create.as_str() {
        "c" => {
            symlink(absolute_original, absolute_link).unwrap();
        }
        "rc" => {
            if let Err(err) = remove_file_dir(link_path) {
                eprintln!("{err}");
            }
            symlink(absolute_original, absolute_link).unwrap();
        }
        _ => todo!(),
    }
    Ok(())
}
