use std::{fs, path, io};
use anyhow::{anyhow, Result};
use io::Read;

fn get_class_name_and_extends_from_line(line: &str) -> Result<(Option<&str>, Option<&str>), &str> {
    if let Some(class_name) = line.trim().strip_prefix("class_name ") {
        let class_name = class_name.trim();
        if !class_name.contains(" ") {
            return Ok((Some(class_name), None));
        }
        let rest_of_line: Vec<&str> = class_name.splitn(1, " ").collect();
        let class_name = rest_of_line[0];
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
pub enum ScriptKind {
    Behaviour,
    Value,
    Reference,
}

#[derive(Debug)]
pub struct Script {
    pub full_name: String,
    pub name: String,
    pub contents: String,
    pub path: path::PathBuf,
    pub kind: ScriptKind,
    pub class_name: Option<String>,
    pub extends: Option<String>,
}

impl Script {
    pub fn new(path: path::PathBuf) -> Result<Self> {
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

    pub fn check_name_is_upper_camel_case(&self) -> Result<()> {
        if let Some(letter) = self.name.get(0..1) {
            if letter != letter.to_uppercase() {
                return Err(anyhow!("{} name not upper camel case", self.full_name));
            }
        }
        if self.name.contains("_") {
            return Err(anyhow!("{} name not upper camel case", self.full_name));
        }
        return Ok(());
    }

    pub fn check_extends_node(&self) -> Result<()> {
        match self.extends.to_owned() {
            Some(extends_inner) => {
                if extends_inner != "Node" {
                    return Err(anyhow!("{} doesn't extend Node", self.full_name));
                }
            }
            None => {
                return Err(anyhow!(
                    "{} doesn't extend anything (should extend Node)",
                    self.full_name
                ));
            }
        };

        return Ok(());
    }

    pub fn check_contains_non_export_var(&self) -> Result<()> {
        for line in self.contents.lines() {
            let trimmed = line.trim();
            if trimmed == ""
                || trimmed.starts_with("class_name")
                || trimmed.starts_with("extends")
                || trimmed.starts_with("#")
            {
                continue;
            }
            if !trimmed.starts_with("@export var ") {
                return Err(anyhow!(
                    "{} contains a non @export var statement",
                    self.full_name
                ));
            }
        }
        return Ok(());
    }

    pub fn check_matching_script_name_and_class_name(&self) -> Result<()> {
        match self.class_name.to_owned() {
            Some(class_name_inner) => {
                if class_name_inner != self.name {
                    return Err(anyhow!(
                        "{} script name and class_name don't match",
                        self.full_name
                    ));
                }
            }
            None => {
                return Err(anyhow!("{} doesn't have a class_name", self.full_name));
            }
        };
        return Ok(());
    }
}