use anyhow::{anyhow, Error, Result};
use std::{env, fs, path};

mod script;
use script::{Script, ScriptKind};

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



fn check_value_script_isolated(script: &Script) -> Result<()> {
    script.check_matching_script_name_and_class_name()?;
    script.check_extends_node()?;
    script.check_contains_non_export_var()?;

    return Ok(());
}

fn check_reference_script_isolated(script: &Script) -> Result<()> {
    if let Some(_) = script.class_name {
        return Err(anyhow!(
            "{} reference script has class_name",
            script.full_name
        ));
    };
    script.check_extends_node()?;
    script.check_contains_non_export_var()?;

    return Ok(());
}

fn check_behaviour_script_isolated(script: &Script) -> Result<()> {
    script.check_matching_script_name_and_class_name()?;
    script.check_extends_node()?;
    return Ok(());
}

fn check_script_isolated(script: &Script) -> Result<()> {
    script.check_name_is_upper_camel_case()?;
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
    match script.kind {
        ScriptKind::Behaviour => check_behaviour_script_isolated(script)?,
        ScriptKind::Reference => check_reference_script_isolated(script)?,
        ScriptKind::Value => check_value_script_isolated(script)?,
    };
    return Ok(());
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
