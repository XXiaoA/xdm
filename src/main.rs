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

fn get_conf() -> Value {
    // get file path from user
    let args = Args::parse();
    let file_path = &args.file;

    let file_content = std::fs::File::open(file_path).expect("Could not open file.");
    let config: Value = serde_yaml::from_reader(file_content).expect("Could not read values.");
    config
}

fn get_link_parameter(conf: Value, original: String, parameter: String) -> String {
    let all_links = conf.get("link").unwrap();
    let all_parameters = all_links.get(original).unwrap();
    let _value = all_parameters.get(&parameter);
    let value = match _value {
        Some(v) => v.as_str().unwrap_or("").to_owned(),
        None => match parameter.as_str() {
            "exist" | "if" | "create" => "true".to_string(),
            "force" => "false".to_string(),
            _ => "".to_string(),
        },
    };
    value
}

fn main() {
    let xdm_config = get_conf();

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
        let link = get_link_parameter(
            xdm_config.to_owned(),
            original.to_string(),
            "path".to_string(),
        );

        if link.is_empty() {
            println!(
                "{}{}",
                original.color("red"),
                ": something wrong in `path`".red()
            )
        } else {
            match create_softlink(original, link.as_str()) {
                Ok(_) => {
                    // TODO: make the following code cleaner
                    // create the parent directory if need
                    let link_path = Path::new(&link);
                    let create: bool = get_link_parameter(
                        xdm_config.to_owned(),
                        original.to_string(),
                        "create".to_string(),
                    )
                    .parse()
                    .unwrap();
                    let link_parent = link_path.parent().unwrap();
                    if create && !link_parent.exists() {
                        fs::create_dir_all(link_parent.to_str().unwrap()).unwrap();
                        create_softlink(original, link.as_str()).unwrap();
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

fn create_softlink(original: &str, link: &str) -> Result<(), String> {
    use std::os::unix::fs::symlink;

    let absolute_original = &absolute_path(original);
    let absolute_link = &absolute_path(link);

    let original_path = Path::new(absolute_original);
    let link_path = Path::new(absolute_link);

    let xdm_config = get_conf();
    let exist: bool = get_link_parameter(
        xdm_config.to_owned(),
        original.to_string(),
        "exist".to_string(),
    )
    .parse()
    .unwrap();
    let force: bool = get_link_parameter(
        xdm_config.to_owned(),
        original.to_string(),
        "force".to_string(),
    )
    .parse()
    .unwrap();
    let condition = get_link_parameter(xdm_config, original.to_string(), "if".to_string());

    let command_status = if condition == "true" {
        true
    } else {
        get_command_status(condition.as_str())
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
