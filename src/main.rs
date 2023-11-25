use anyhow::{anyhow, Error, Result};
use std::{env, fs, io, path};

use io::Read;

fn visit_dirs<const N: usize>(
    dir: &path::Path,
    target_extension: &str,
    skip_dirs: [&str; N],
) -> Result<Vec<path::PathBuf>> {
    let mut files = Vec::new();

    for skip_dir in skip_dirs {
        if dir.ends_with(skip_dir) {
            return Ok(files);
        }
    }

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively visit subdirectories
                let mut subdirectory_files = visit_dirs(&path, target_extension, skip_dirs)?;
                files.append(&mut subdirectory_files);
            } else if let Some(extension) = path.extension() {
                // Check if the file has the target extension
                if extension == target_extension {
                    files.push(path);
                }
            }
        }
    }

    Ok(files)
}

#[derive(Debug)]
enum ScriptKind {
    Behaviour,
    Value,
    Reference,
}

#[derive(Debug)]
struct Script {
    full_name: String,
    name: String,
    contents: String,
    path: path::PathBuf,
    kind: ScriptKind,
}

impl Script {
    fn new(path: path::PathBuf) -> Result<Self> {
        let mut file = fs::File::open(path.clone())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let full_name = path
            .file_name()
            .ok_or(anyhow!("no file name for file {}", path.display()))?
            .to_string_lossy()
            .to_string();

        let kind = match full_name.get(0..2) {
            Some("v_") => ScriptKind::Value,
            Some("r_") => ScriptKind::Reference,
            _ => ScriptKind::Behaviour,
        };

        let name = match kind {
            ScriptKind::Behaviour => full_name.clone(),
            _ => full_name
                .get(2..)
                .ok_or(anyhow!("this shouldn't happen"))?
                .to_string(),
        };

        return Ok(Self {
            full_name,
            name,
            path,
            contents,
            kind,
        });
    }
}

fn is_upper_camel_case(name: &String) -> bool {
    if let Some(letter) = name.get(0..1) {
        if letter != letter.to_uppercase() {
            return false;
        }
    }
    if name.contains("_") {
        return false;
    }
    return true;
}

fn check_value_script_isolated(script: &Script) -> Result<()> {
    return Ok(());
}

fn check_reference_script_isolated(script: &Script) -> Result<()> {
    return Ok(());
}

fn check_behaviour_script_isolated(script: &Script) -> Result<()> {
    let mut lines = script.contents.lines();
    let class_def = (lines.next(), lines.next());
    match class_def.0 {
        Some(cd0) => {
            if !cd0.starts_with("class_name") {
                return Err(anyhow!("{} doesn't have a class_name", script.full_name));
            }
        }
        None => {
            return Err(anyhow!(
                "{} doesn't have a first line for class_name",
                script.full_name
            ))
        }
    }
    match class_def.1 {
        Some(cd1) => {
            if !cd1.starts_with("extends Node") {
                return Err(anyhow!(
                    "{} doesn't extend Node, Node2D or Node3D",
                    script.full_name
                ));
            }
        }
        None => {
            return Err(anyhow!(
                "{} doesn't have a second line for extends",
                script.full_name
            ))
        }
    }
    return Ok(());
}

fn check_script_isolated(script: &Script) -> Result<()> {
    if !is_upper_camel_case(&script.name) {
        return Err(anyhow!("{} name not upper camel case", script.full_name));
    }
    return match script.kind {
        ScriptKind::Behaviour => check_value_script_isolated(script),
        ScriptKind::Reference => check_reference_script_isolated(script),
        ScriptKind::Value => check_behaviour_script_isolated(script),
    };
}

fn main() -> Result<()> {
    println!("Hello, godot checker!");
    let args: Vec<String> = env::args().collect();
    // let path = path::Path::new(args.get(1).ok_or(anyhow!("couldn't read path"))?);

    let path = path::Path::new(r"C:\Users\jonathan\files\godot-projects\pushgame");

    let script_paths: Vec<path::PathBuf> = visit_dirs(path, "gd", [".godot", "addons"]).unwrap();

    let mut scripts: Vec<Script> = Vec::new();

    for script_path in script_paths {
        scripts.push(Script::new(script_path)?);
    }

    let problems: Vec<Error> = scripts
        .iter()
        .filter_map(|script| {
            if let Err(e) = check_script_isolated(script) {
                Some(e)
            } else {
                None
            }
        })
        .collect();

    dbg!(problems);

    Ok(())
}
