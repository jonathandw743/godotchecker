use anyhow::{anyhow, Error, Result};
use std::{env, fs, io, path};

use io::Read;

fn split_on_any<'a>(s: &'a String, patterns: &[String]) -> Vec<&'a str> {
    let mut result = vec![s.get(..).unwrap()];
    for pattern in patterns {
        result = result
            .iter()
            .map(|s| s.split(pattern).collect::<Vec<&str>>())
            .flatten()
            .collect::<Vec<&str>>();
    }
    return result;
}

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

fn get_class_name_and_extends_from_line(line: &str) -> Result<(Option<&str>, Option<&str>), &str> {
    if let Some(class_name) = line.trim().strip_prefix("class_name") {
        let class_name = class_name.trim();
        if !class_name.contains(" ") {
            return Ok((Some(class_name), None));
        }
        let rest_of_line: Vec<&str> = class_name.splitn(1, " ").collect();
        let class_name = rest_of_line[0];
        dbg!(rest_of_line.clone());
        if let Some(extends) = rest_of_line[1].strip_prefix("extends ") {
            return Ok((Some(class_name), Some(extends)));
        }
        return Err("there was another word after the class_name but no extends");
    }
    if let Some(extends) = line.trim().strip_prefix("extends") {
        let extends = extends.trim();
        if !extends.contains(" ") {
            return Ok((None, Some(extends)));
        }
        return Err("there was another word after the extends");
    }
    return Ok((None, None));
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
    class_name: Option<String>,
    extends: Option<String>,
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
            ScriptKind::Behaviour => path
                .file_stem()
                .ok_or(anyhow!("no file stem for file {}", path.display()))?
                .to_string_lossy()
                .to_string(),
            _ => path
                .file_stem()
                .ok_or(anyhow!("no file stem for file {}", path.display()))?
                .to_string_lossy()
                .to_string()
                .get(2..)
                .ok_or(anyhow!("this shouldn't happen"))?
                .to_string(),
        };

        let mut class_name = None;
        let mut extends = None;
        for line in contents.lines() {
            let temp = get_class_name_and_extends_from_line(line);
            match temp {
                Err(message) => {
                    return Err(anyhow!("{} {}", full_name, message));
                }
                Ok(options) => {
                    if let Some(class_name_result) = options.0 {
                        if let Some(_) = class_name {
                            return Err(anyhow!("{} multiple class_names", full_name));
                        }
                        class_name = Some(class_name_result.to_string());
                    }
                    if let Some(extends_result) = options.1 {
                        if let Some(_) = extends {
                            return Err(anyhow!("{} multiple extends", full_name));
                        }
                        extends = Some(extends_result.to_string());
                    }
                }
            }
        }

        dbg!(full_name.clone(), class_name.clone(), extends.clone());

        return Ok(Self {
            full_name,
            name,
            path,
            contents,
            kind,
            class_name,
            extends,
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
    match script.class_name.to_owned() {
        Some(class_name_inner) => {
            if class_name_inner != script.name {
                return Err(anyhow!(
                    "{} script name and class_name don't match",
                    script.full_name
                ));
            }
        }
        None => {
            return Err(anyhow!("{} doesn't have a class_name", script.full_name));
        }
    }
    match script.extends.to_owned() {
        Some(extends_inner) => {
            if extends_inner != "Node" {
                return Err(anyhow!(
                    "{} doesn't extend Node",
                    script.full_name
                ));
            }
        }
        None => {
            return Err(anyhow!(
                "{} doesn't have an extends (should extend Node)",
                script.full_name
            ));
        }
    }

    for line in script.contents.lines() {
        if line.trim_start().starts_with("class_name") {
            continue;
        }
        if line.trim_start().starts_with("extends") {
            continue;
        }
        if line.trim_start().starts_with("#") {
            continue;
        }
        let trimmed = line.trim();
        if trimmed == "" {
            continue;
        }
        dbg!(trimmed);
        if !trimmed.starts_with("@export var ") {
            return Err(anyhow!(
                "{} value script contains a non @export var statement",
                script.full_name
            ));
        }
    }

    return Ok(());
}

fn check_reference_script_isolated(script: &Script) -> Result<()> {
    if let Some(_) = script.class_name {
        return Err(anyhow!(
            "{} reference script has class_name",
            script.full_name
        ));
    };

    match script.extends.to_owned() {
        Some(extends_inner) => {
            if extends_inner != "Node" {
                return Err(anyhow!(
                    "{} reference script does not extend Node",
                    script.full_name
                ));
            }
        }
        None => {
            return Err(anyhow!(
                "{} reference script does not extend anything (should extend node)",
                script.full_name
            ));
        }
    }

    for line in script.contents.lines() {
        if line.trim_start().starts_with("class_name") {
            continue;
        }
        if line.trim_start().starts_with("extends") {
            continue;
        }
        if line.trim_start().starts_with("#") {
            continue;
        }
        let trimmed = line.trim();
        if trimmed == "" {
            continue;
        }
        if !trimmed.starts_with("@export var ") {
            return Err(anyhow!(
                "{} reference script contains a non @export var statement",
                script.full_name
            ));
        }
    }

    return Ok(());
}

fn check_behaviour_script_isolated(script: &Script) -> Result<()> {
    match script.class_name.to_owned() {
        Some(class_name_inner) => {
            if class_name_inner != script.name {
                return Err(anyhow!(
                    "{} script name and class_name don't match",
                    script.full_name
                ));
            }
        }
        None => {
            return Err(anyhow!("{} doesn't have a class_name", script.full_name));
        }
    }
    match script.extends.to_owned() {
        Some(extends_inner) => {
            if extends_inner != "Node" {
                return Err(anyhow!(
                    "{} doesn't extend Node",
                    script.full_name
                ));
            }
        }
        None => {
            return Err(anyhow!(
                "{} doesn't have an extends (should extend Node)",
                script.full_name
            ));
        }
    }
    return Ok(());
}

fn check_script_isolated(script: &Script) -> Result<()> {
    if !is_upper_camel_case(&script.name) {
        return Err(anyhow!("{} name not upper camel case", script.full_name));
    }
    if script.contents.contains("$") {
        return Err(anyhow!("{} contains $", script.full_name));
    }
    for section in split_on_any(&script.contents, &[" var ".into(), "\tvar ".into()]) {
        for letter in section.chars() {
            if letter == ':' {
                break;
            }
            if letter == '=' {
                return Err(anyhow!(
                    "{} doesn't use static typing properly",
                    script.full_name
                ));
            }
        }
    }
    return match script.kind {
        ScriptKind::Behaviour => check_behaviour_script_isolated(script),
        ScriptKind::Reference => check_reference_script_isolated(script),
        ScriptKind::Value => check_value_script_isolated(script),
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
