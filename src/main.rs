use directories::ProjectDirs;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use text_io::read;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::CompressionMethod;

fn main() {
    let mut templates_path = ProjectDirs::from("", "IndustrialCraft", "Templates")
        .unwrap()
        .data_dir()
        .to_owned();
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
            "export" => action_export(&templates_path, argument).unwrap(),
            "import" => action_import(&templates_path, argument).unwrap(),
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
fn action_export(templates_path: &PathBuf, argument: Option<String>) -> Result<(), Box<dyn Error>> {
    if let Some(argument) = argument {
        let regex: Regex = Regex::new(argument.as_str())?;
        let entries: Vec<PathBuf> = fs::read_dir(templates_path)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.metadata().unwrap().is_dir() && regex.is_match(e.file_name().to_str().unwrap())
            })
            .map(|e| e.path())
            .collect();
        fs::remove_file("export.zip").ok();
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("export.zip")
            .unwrap();
        zip_dir(entries, templates_path, file, CompressionMethod::Deflated)?;
    } else {
        println!("USAGE: template export <regex>");
    }
    Ok(())
}
fn action_import(templates_path: &PathBuf, argument: Option<String>) -> Result<(), Box<dyn Error>> {
    if let Some(argument) = argument {
        let file = File::open(&argument);
        if let Ok(file) = file {
            let mut archive = zip::ZipArchive::new(file).unwrap();
            for i in 0..archive.len() {
                let mut path = templates_path.clone();
                let mut file = archive.by_index(i).unwrap();
                let outpath = match file.enclosed_name() {
                    Some(path) => path.to_owned(),
                    None => continue,
                };
                if outpath.parent().unwrap().parent().is_none() {
                    let mut template_path = templates_path.clone();
                    template_path.push(&outpath.file_name().unwrap().to_str().unwrap().to_owned());
                    fs::remove_dir_all(template_path).ok();
                }
                path.push(outpath);
                if (*file.name()).ends_with('/') {
                    fs::create_dir_all(&path).unwrap();
                } else {
                    if let Some(p) = path.parent() {
                        if !p.exists() {
                            fs::create_dir_all(&p).unwrap();
                        }
                    }
                    let mut outfile = File::create(&path).unwrap();
                    io::copy(&mut file, &mut outfile).unwrap();
                }
            }
        } else {
            println!("Archive not found");
        }
    } else {
        println!("USAGE: template export <file>");
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
                    .strip_prefix(&template_path)
                    .unwrap()
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .to_string();
                if !rel_path.is_empty() {
                    entries.push((rel_path, entry.path().to_owned()));
                }
            }
        }
        entries.sort_by(|e1, e2| e1.0.len().cmp(&e2.0.len()));
        let mut replacements = HashMap::new();
        for e in &entries {
            for replacement in extract_replacements(&e.0) {
                if !replacements.contains_key(replacement.as_str()) {
                    print!("{}: ", replacement);
                    let line: String = read!("{}\n");
                    replacements.insert(replacement, line);
                }
            }
            if e.1.is_file() {
                for replacement in extract_replacements(&fs::read_to_string(&e.1).unwrap()) {
                    if !replacements.contains_key(replacement.as_str()) {
                        print!("{}: ", replacement);
                        let line: String = read!("{}\n");
                        replacements.insert(replacement, line);
                    }
                }
            }
        }
        for e in &entries {
            let mut path = env::current_dir().unwrap();
            let path_tmp =
                Path::new(replace_replacements(e.0.as_str().to_string(), &replacements).as_str())
                    .to_owned();
            if path_tmp.is_absolute() {
                panic!("Something went wrong and template path was absolute");
            }
            path.push(path_tmp);
            if e.1.is_dir() {
                fs::create_dir(path).unwrap();
            } else {
                let file_content = fs::read_to_string(&e.1).unwrap();
                fs::write(&path, replace_replacements(file_content, &replacements)).unwrap();
            }
        }
    } else {
        println!("USAGE: template use <name>");
    }
    Ok(())
}
lazy_static! {
    static ref REPLACEMENT_REGEX: Regex = Regex::new(r"§%\{([\w_.]+)\}").unwrap();
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
    println!("\ttemplate export <regex>: Exports template to zip");
    println!("\ttemplate import <file>: Imports template from zip");
}
fn zip_dir(
    it: Vec<PathBuf>,
    prefix: &Path,
    writer: File,
    method: CompressionMethod,
) -> zip::result::ZipResult<()> {
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry_dir in it {
        for entry in WalkDir::new(entry_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_owned())
        {
            let path = entry;
            let name = path.strip_prefix(Path::new(prefix)).unwrap();

            if path.is_file() {
                #[allow(deprecated)]
                zip.start_file_from_path(name, options)?;
                let mut f = File::open(path)?;

                f.read_to_end(&mut buffer)?;
                zip.write_all(&*buffer)?;
                buffer.clear();
            } else if !name.as_os_str().is_empty() {
                #[allow(deprecated)]
                zip.add_directory_from_path(name, options)?;
            }
        }
    }
    zip.finish()?;
    Ok(())
}
