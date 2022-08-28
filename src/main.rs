use clap::Parser;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::{collections::HashMap, path::Path};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File path to YAMl file
    #[clap(default_value_t = String::from("xdm.yaml") ,value_parser)]
    file: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
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
                "exist" => "true",
                "force" => "false",
                "if" => "true",
                _ => "",
            });
        value
    }
}

fn main() {
    let xdm_config = Config::get_conf();
    let all_links = &xdm_config.link;
    for link_item in all_links {
        let original = link_item.0;
        let path = xdm_config.get_link_parameter(original, "path");
        if path.is_empty() {
            println!(
                "{}{}",
                original.color("red"),
                ": something wrong in `path`".red()
            )
        } else {
            match create_softlink(original, path) {
                Ok(_) => println!(
                    "{}{}",
                    original.color("green"),
                    ": created link successfully".green()
                ),
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
    use std::fs;
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

fn create_softlink(original: &str, link: &str) -> Result<(), String> {
    use std::os::unix::fs::symlink;

    let original_path = Path::new(original);
    let link_path = Path::new(link);

    let xdm_config = Config::get_conf();
    let exist = xdm_config.get_link_parameter(original, "exist");
    let force = xdm_config.get_link_parameter(original, "force");
    let condition = xdm_config.get_link_parameter(original, "if");

    let command_status = if condition == "true" {
        true
    } else {
        get_command_status(condition)
    };

    if !command_status {
        Err(String::from("skip to create link"))
    } else if exist == "false" && force == "true" {
        remove_file_dir(link_path).err();
        symlink(original, link).err();
        Ok(())
    } else if exist == "false" && !Path::exists(link_path) {
        symlink(original, link).err();
        Ok(())
    } else if !Path::exists(original_path) && force == "true" {
        symlink(original, link).err();
        Ok(())
    } else if !Path::exists(original_path) {
        Err(String::from(format!(
            "the file `{}` doesn't exist",
            original
        )))
    } else {
        Err(String::from("skip to create link"))
    }
}
