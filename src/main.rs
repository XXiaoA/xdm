use clap::Parser;
use colored::Colorize;
use serde_yaml::Value;
use std::{fs, path::Path};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File path to YAMl file
    #[clap(default_value_t = String::from("xdm.yaml") ,value_parser)]
    file: String,
}

trait Configuration {
    fn get_link_parameter(&self, original: &str, parameter: &str) -> &str;
}

impl Configuration for Value {
    fn get_link_parameter(&self, original: &str, parameter: &str) -> &str {
        let all_links = self.get("link").unwrap();
        let all_parameters = all_links.get(original).unwrap();
        let _value = all_parameters.get(&parameter);
        let value = if let Some(v) = _value {
            if v.is_bool() {
                match v {
                    Value::Bool(true) => "true",
                    Value::Bool(false) => "false",
                    _ => "",
                }
            } else {
                v.as_str().unwrap_or("")
            }
        } else {
            match parameter {
                "exist" | "if" | "create" => "true",
                "force" => "false",
                _ => "",
            }
        };

        value
    }
}

fn get_conf() -> Value {
    // get file path from user
    let args = Args::parse();
    let file_path = &args.file;
    if Path::new(file_path).exists() {
        let file_content = std::fs::File::open(file_path).expect("Could not open file.");
        serde_yaml::from_reader(file_content).expect("Could not read values.")
    } else {
        let yaml = "Can't find configuration file";
        serde_yaml::from_str(yaml).unwrap()
    }
}

fn main() {
    let xdm_config = get_conf();
    if xdm_config == "Can't find configuration file" {
        println!("{}", "Can't find configuration file".bold().red());
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
    println!("#######link#######");
    let all_link_items = xdm_config.get("link").unwrap().as_mapping().unwrap();
    for link_item in all_link_items {
        let original = link_item.0.as_str().unwrap();
        let link = xdm_config.get_link_parameter(original, "path");

        if link.is_empty() {
            println!(
                "{}{}",
                original.color("red"),
                ": something wrong in `path`".red()
            )
        } else {
            match create_softlink(original, link) {
                Ok(_) => {
                    println!("{} {} {}", original.green(), "=>".green(), link.green());
                }
                Err(err) => println!("{}", format!("{}: {}", original, err).blue()),
            }
        }
    }
}

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

fn remove_file_dir(path: &Path) -> Result<(), String> {
    if !Path::exists(path) {
        Err(String::from("The path doesn't exist"))
    } else {
        if Path::is_dir(path) {
            fs::remove_dir_all(path).unwrap();
        } else if Path::is_file(path) {
            fs::remove_file(path).unwrap();
        }
        Ok(())
    }
}

fn absolute_path(path: &str) -> String {
    use path_absolutize::Absolutize;
    use std::env;

    let path_vec: Vec<&str> = path.split('/').collect();

    if path_vec[0] == "~" {
        let prev = env::var("HOME").unwrap_or_else(|_| "none".to_string());
        let end = path_vec[1..].join("/");
        format!("{}/{}", prev, end)
    } else if &path_vec[0][0..1] == "$" {
        let env = &path_vec[0][1..];
        let prev = env::var(env).unwrap_or_else(|_| "none".to_string());
        let end = path_vec[1..].join("/");
        format!("{}/{}", prev, end)
    } else {
        let absolute_path = Path::new(path).absolutize().unwrap();
        absolute_path.to_str().unwrap().to_string()
    }
}

// TODO: add support for "a: b"
fn create_softlink(original: &str, link: &str) -> Result<(), String> {
    use std::os::unix::fs::symlink;

    fn can_create(original: &str, link: &str) -> Result<String, String> {
        let absolute_original = &absolute_path(original);
        let absolute_link = &absolute_path(link);

        let original_path = Path::new(absolute_original);
        let link_path = Path::new(absolute_link);

        let xdm_config = get_conf();
        let exist: bool = xdm_config
            .get_link_parameter(original, "exist")
            .parse()
            .unwrap();
        let force: bool = xdm_config
            .get_link_parameter(original, "force")
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
            // remove_file_dir(link_path).unwrap();
            // symlink(absolute_original, absolute_link).unwrap();
            Ok(String::from("rc"))
        } else if !exist && !link_path.exists() {
            // symlink(absolute_original, absolute_link).unwrap();
            Ok(String::from("c"))
        } else if original_path.exists() && force {
            // remove_file_dir(link_path).unwrap();
            // symlink(absolute_original, absolute_link).unwrap();
            Ok(String::from("rc"))
        } else if !original_path.exists() {
            Err(format!("the file `{}` doesn't exist", original))
        } else {
            // symlink(absolute_original, absolute_link).err();
            Ok(String::from("c"))
        }
    }

    let can_create = can_create(original, link);

    if can_create.is_ok() {
        let absolute_original = &absolute_path(original);
        let absolute_link = &absolute_path(link);
        let link_path = Path::new(absolute_link);
        let link_parent = link_path.parent().unwrap();

        let create: bool = get_conf()
            .get_link_parameter(original, "create")
            .parse()
            .unwrap();

        if create && !link_parent.exists() {
            fs::create_dir_all(link_parent.to_str().unwrap()).unwrap();
        }
        match can_create.ok().unwrap().as_str() {
            "c" => {
                symlink(absolute_original, absolute_link).err();
            }
            "rc" => {
                remove_file_dir(link_path).unwrap();
                symlink(absolute_original, absolute_link).unwrap();
            }
            _ => todo!(),
        }

        Ok(())
    } else {
        Err(can_create.err().unwrap())
    }
}
