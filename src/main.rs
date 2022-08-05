use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{env, fs};
use text_io::read;
use walkdir::WalkDir;

fn main() {
    let mut templates_path = env::current_dir().unwrap();
    templates_path.push(".templates");
    fs::create_dir(&templates_path).ok();

    let mut args = env::args();
    let action = args.nth(1);
    let argument = args.next();
    if let Some(_) = args.next() {
        show_usage();
        return;
    }
    if let Some(action) = action {
        match action.as_str() {
            "list" => action_list(&templates_path, argument).unwrap(),
            "use" => {
                action_use(&templates_path, argument).unwrap();
            }
            "create" => {
                action_create(&templates_path, argument).unwrap();
            }
            "remove" => action_remove(&templates_path, argument).unwrap(),
            _ => show_usage(),
        }
    } else {
        show_usage();
    }
}
fn action_list(templates_path: &PathBuf, argument: Option<String>) -> Result<(), Box<dyn Error>> {
    if let None = argument {
        println!("INSTALLED TEMPLATES:");
        for entry in fs::read_dir(templates_path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                println!("\t{}", entry.path().file_name().unwrap().to_str().unwrap());
            }
        }
    } else {
        println!("USAGE: template list");
    }
    Ok(())
}
fn action_remove(templates_path: &PathBuf, argument: Option<String>) -> Result<(), Box<dyn Error>> {
    if let Some(argument) = argument {
        let mut template_path = templates_path.clone();
        template_path.push(&argument);
        if let Err(_) = fs::remove_dir_all(template_path) {
            println!("Template {} doesnt exist", &argument);
        }
    } else {
        println!("USAGE: template remove <name>");
    }
    Ok(())
}
fn action_create(templates_path: &PathBuf, argument: Option<String>) -> Result<(), Box<dyn Error>> {
    if let Some(argument) = argument {
        let mut template_path = templates_path.clone();
        template_path.push(&argument);
        if template_path.exists() {
            println!("template {} already exists", argument);
        } else {
            fs::create_dir(&template_path).ok();
        }
        println!("its path is: {}", template_path.to_str().unwrap());
        opener::open(Path::new(&template_path))?;
    } else {
        println!("USAGE: template create <name>");
    }
    Ok(())
}
fn action_use(templates_path: &PathBuf, argument: Option<String>) -> Result<(), Box<dyn Error>> {
    if let Some(argument) = argument {
        let mut template_path = templates_path.clone();
        template_path.push(&argument);
        if (!template_path.exists()) || (!template_path.metadata()?.is_dir()) {
            println!("template {} doesnt exist", argument);
            return Ok(());
        }
        let mut entries = Vec::new();
        for entry in WalkDir::new(&template_path) {
            if let Ok(entry) = entry {
                let rel_path = entry
                    .path()
                    .strip_prefix(&template_path)?
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .to_string();
                if !rel_path.is_empty() {
                    entries.push(rel_path);
                }
            }
        }
        entries.sort_by(|e1, e2| e1.len().cmp(&e2.len()));
        let mut replacements = HashMap::new();
        for e in &entries {
            for replacement in extract_replacements(e) {
                if !replacements.contains_key(replacement.as_str()) {
                    print!("{}: ", replacement);
                    let line: String = read!("{}\n");
                    replacements.insert(replacement, line);
                    println!();
                }
            }
        }
        for e in &entries {
            let mut path = env::current_dir()?;
            let path_tmp =
                Path::new(replace_replacements(e.as_str().to_string(), &replacements).as_str())
                    .to_owned();
            if path_tmp.is_absolute() {
                panic!("Something went wrong and template path was absolute");
            }
            path.push(path_tmp);
            if path.is_dir() {
                fs::create_dir(path)?;
            } else {
                fs::
            }
        }
    } else {
        println!("USAGE: template use <name>");
    }
    Ok(())
}
lazy_static! {
    static ref REPLACEMENT_REGEX: Regex = Regex::new(r"§%{([\w_.]+)}").unwrap();
}
fn extract_replacements(input: &String) -> Vec<String> {
    REPLACEMENT_REGEX
        .captures_iter(input.as_str())
        .map(|s| s.get(1).unwrap().as_str().to_string())
        .collect()
}
fn replace_replacements(input: String, replacements: &HashMap<String, String>) -> String {
    let mut output = input;
    for replacement in replacements {
        output = output.replace(
            format!("§%{{{}}}", replacement.0).as_str(),
            replacement.1.as_str(),
        );
    }
    output
}
fn show_usage() {
    println!("USAGE:");
    println!("\ttemplate list: Lists all installed templates");
    println!("\ttemplate use <name>: Uses template");
    println!("\ttemplate create <name>: Creates template and opens its folder");
    println!("\ttemplate remove <name>: Removes template");
    println!("\ttemplate export <file>: Exports template to zip");
    println!("\ttemplate imports <file>: Imports template from zip");
}
