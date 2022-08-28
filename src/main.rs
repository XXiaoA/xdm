use clap::Parser;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File path to YAMl file
    #[clap(default_value_t = String::from("xdm.yaml") ,value_parser)]
    file: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    create: Vec<String>,
    link: HashMap<String, HashMap<String, String>>,
}

impl Config {
    fn get_conf() -> Config {
        // get file path from user
        let args = Args::parse();
        let file_path = &args.file;

        let file_content = std::fs::File::open(file_path).expect("Could not open file.");
        let config: Config = serde_yaml::from_reader(file_content).expect("Could not read values.");
        config
    }

    fn get_link_parameter(&self, original: &str, parameter: &str) -> &str {
        let all_links = &self.link;
        let all_parameters = all_links.get(original).unwrap();
        let value = all_parameters
            .get(parameter)
            .map(String::as_str)
            .unwrap_or_else(|| match parameter {
                "exist" | "if" | "create" => "true",
                "force" => "false",
                _ => "",
            });
        value
    }
}

fn main() {
    let xdm_config = Config::get_conf();
    // do jobs for create
    println!("######job#######");
    let all_creates = &xdm_config.create;
    for directory in all_creates {
        let path = absolute_path(directory);
        let path = Path::new(&path);
        if !path.exists() {
            fs::create_dir_all(path).unwrap();
            println!("{}{}", directory.green(), ": created the directory successfully".green())
        }
    }

    // do jobs for link
    println!("######link######");
    let all_links = &xdm_config.link;
    for link_item in all_links {
        let original = link_item.0;
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
                    // TODO: make the following code cleaner
                    // create the parent directory if need
                    let link_path = Path::new(link);
                    let create: bool = xdm_config
                        .get_link_parameter(original, "create")
                        .parse()
                        .unwrap();
                    let link_parent = link_path.parent().unwrap();
                    if create && !link_parent.exists() {
                        fs::create_dir_all(link_parent.to_str().unwrap()).unwrap();
                        create_softlink(original, link).unwrap();
                    }
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

    let path_vec: Vec<&str> = path.split("/").collect();

    if path_vec[0] == "~" {
        let prev = env::var("HOME").unwrap_or("none".to_string());
        let end = path_vec[1..].join("/");
        format!("{}/{}", prev, end)
    } else if &path_vec[0][0..1] == "$" {
        let env = &path_vec[0][1..];
        let prev = env::var(env).unwrap_or("none".to_string());
        let end = path_vec[1..].join("/");
        format!("{}/{}", prev, end)
    } else {
        let absolute_path = Path::new(path).absolutize().unwrap();
        absolute_path.to_str().unwrap().to_string()
    }
}

fn create_softlink(original: &str, link: &str) -> Result<(), String> {
    use std::os::unix::fs::symlink;

    let absolute_original = &absolute_path(original);
    let absolute_link = &absolute_path(link);

    let original_path = Path::new(absolute_original);
    let link_path = Path::new(absolute_link);

    let xdm_config = Config::get_conf();
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
        remove_file_dir(link_path).unwrap();
        symlink(absolute_original, absolute_link).unwrap();
        Ok(())
    } else if (!exist && !link_path.exists()) || (!original_path.exists() && force) {
        symlink(absolute_original, absolute_link).unwrap();
        Ok(())
    } else if !original_path.exists() {
        Err(format!("the file `{}` doesn't exist", original))
    } else {
        symlink(absolute_original, absolute_link).err();
        Ok(())
    }
}
