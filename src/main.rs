use clap::Parser;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;

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
}

fn main() {
    let xdm_config = Config::get_conf();
    let all_links = xdm_config.link;
    // println!("{:#?}", all_links);
    for link_item in all_links.iter() {
        let original = link_item.0;
        let parameter = link_item.1;
        let link = parameter.get("path");
        if let Some(path) = link {
            if path.is_empty() {
                println!(
                    "{}{}",
                    original.color("red"),
                    ": 'path' parameter mustn't be empty".color("red")
                )
            } else {
                // FIX: should check link was created successfully or nor <XXiaoA>
                create_softlink(original, path);
                println!(
                    "{}{}",
                    original.color("green"),
                    ": created link successfully".color("green")
                )
            }
        } else {
            println!(
                "{}{}",
                original.color("red"),
                ": must have 'path' parameter".color("red")
            );
        }
    }
}

fn create_softlink(original: &String, link: &String) {
    use std::os::unix;
    use std::path::Path;

    let xdm_config = Config::get_conf();
    let parameters = xdm_config.link.get(original).unwrap();
    let exist = parameters
        .get("exist")
        .map(String::as_str)
        .unwrap_or("true");

    if exist == "true" && Path::exists(Path::new(original)) && !Path::is_symlink(Path::new(link)) {
        unix::fs::symlink(original, link).unwrap_or_else(|err| println!("{:?}", err));
    } else if exist == "false" && !Path::is_symlink(Path::new(link)) {
        unix::fs::symlink(original, link).unwrap_or_else(|err| println!("{:?}", err));
    }
}
