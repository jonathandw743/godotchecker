mod script;
mod scene;

use anyhow::{anyhow, Error, Result};
use std::{env, fs, path};

use script::{Script, ScriptKind};
use scene::Scene;

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

fn check_scene(scene: &Scene) -> Result<()> {
    if scene.has_children && scene.script.is_some() {
        return Err(anyhow!("{} scene has children and script", scene.full_name));
    }
    if !["Node".into(), "Node2D".into(), "Node3D".into()].contains(&scene.gd_type) {
        return Err(anyhow!("{} scene master node is not of type Node, Node2D, or Node3D", scene.full_name));
    }
    if let Some(script) = scene.script {
        match script.kind {
            ScriptKind::Behaviour | ScriptKind::Value => {
                if scene.gd_type != "Node" {
                    return Err(anyhow!("{} isn't type Node but has a behaviour or value script", scene.full_name));
                }
            },
            ScriptKind::Reference => {
                return Err(anyhow!("{} reference script should not be on scene master", scene.full_name));
            }
        }
    }
    return Ok(());
}

fn main() -> Result<()> {
    println!("Hello, godot checker!");
    // let args: Vec<String> = env::args().collect();
    // let path = path::Path::new(args.get(1).ok_or(anyhow!("couldn't read path"))?);

    let path = path::Path::new(r"C:\Users\jonathan\files\godot-projects\pushgame");

    // get the paths of all .gd files
    let script_paths: Vec<path::PathBuf> = visit_dirs(path, "gd", [".godot", "addons"]).unwrap();
    // get the relative path to each .gd file
    let rel_script_paths: Vec<&path::Path> = script_paths
        .iter()
        .filter_map(|script_path| match script_path.strip_prefix(path) {
            Ok(rel_path) => Some(rel_path),
            Err(_) => None,
        })
        .collect();
    if rel_script_paths.len() < script_paths.len() {
        return Err(anyhow!("something went wrong when finding relative paths"));
    }

    // get godot's name for each script 
    let gd_script_paths: Vec<String> = rel_script_paths
        .iter()
        .map(|rel_script_path| {
            format!(
                "res://{}",
                rel_script_path.to_string_lossy().replace(r"\", r"/")
            )
        })
        .collect();

    let mut general_problems: Vec<Error> = Vec::new();
    
    // create Script sctructs
    let mut scripts: Vec<Script> = Vec::new();
    for (script_path, gd_script_path) in script_paths.iter().zip(gd_script_paths.iter()) {
        let script = Script::new(
            script_path.to_owned(),
            gd_script_path.to_owned(),
        );
        match script {
            Ok(script) => {
                scripts.push(script);
            },
            Err(e) => {
                general_problems.push(e);
            }
        }
    }

    // get the problems with the scripts in isolation
    let script_problems: Vec<Error> = scripts
        .iter()
        .filter_map(|script| {
            if let Err(e) = check_script_isolated(script) {
                return Some(e);
            }
            return None;
        })
        .collect();

    // get the paths of all .tscn files
    let scene_paths: Vec<path::PathBuf> = visit_dirs(path, "tscn", [".godot", "addons"]).unwrap();
    
    // create scene structs
    let mut scenes: Vec<Scene> = Vec::new();
    for scene_path in scene_paths {
        let scene = Scene::new(scene_path, &scripts);
        match scene {
            Ok(script) => {
                scenes.push(script);
            },
            Err(e) => {
                general_problems.push(e);
            }
        }
    }
    
    // get the problems with the scenes
    let scene_problems: Vec<Error> = scenes
        .iter()
        .filter_map(|scene| {
            if let Err(e) = check_scene(scene) {
                return Some(e);
            }
            return None;
        })
        .collect();

    dbg!(general_problems);
    dbg!(script_problems);
    dbg!(scene_problems);

    Ok(())
}
