use anyhow::{anyhow, Result};
use io::Read;
use std::{fs, io, path};

use crate::script::Script;

pub struct Scene<'a> {
    pub full_name: String,
    pub name: String,
    pub path: path::PathBuf,
    pub contents: String,
    pub script: Option<&'a Script>,
    pub has_children: bool,
    pub gd_type: String,
}

impl<'a> Scene<'a> {
    pub fn new(path: path::PathBuf, scripts: &'a Vec<Script>) -> Result<Self> {
        let mut file = fs::File::open(path.clone())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let full_name = path
            .file_name()
            .ok_or(anyhow!("no file name for file {}", path.display()))?
            .to_string_lossy()
            .to_string();

        let name = path
            .file_stem()
            .ok_or(anyhow!("no file stem for file {}", path.display()))?
            .to_string_lossy()
            .to_string();

        let gd_script_path = contents
            .lines()
            .find_map(|line| {
                line.strip_prefix("[ext_resource type=\"Script\" path=\"")
                    .map(|script_def_eol| {
                        script_def_eol.splitn(1, '"').next().ok_or(anyhow!(
                            "{} no ending \" on script ext_resource path",
                            full_name
                        ))
                    })
            })
            .transpose()?;

        let script = gd_script_path
            .map(|gd_script_path| {
                scripts
                    .iter()
                    .find(|script| script.gd_path == gd_script_path)
            })
            .flatten();

        let mut num_nodes = 0;
        let mut gd_type_option = None;
        for line in contents.lines() {
            if line.starts_with("[node ") {
                if num_nodes == 0 {
                    let start_of_type_pattern = "type=\"";
                    let start_of_gd_type = line
                        .find(start_of_type_pattern)
                        .ok_or(anyhow!("{} no type on master node", full_name))?;
                    let gd_type_intermediate = line
                        .get((start_of_gd_type + start_of_type_pattern.len())..)
                        .ok_or(anyhow!("{} can't get type on master node", full_name))?;
                    let end_of_gd_type = gd_type_intermediate.find('"').ok_or(anyhow!(
                        "{} can't find end of type on master node",
                        full_name
                    ))?;
                    gd_type_option = Some(
                        gd_type_intermediate
                            .get(..end_of_gd_type)
                            .ok_or(anyhow!("{} can't get type on master node", full_name))?,
                    );
                }
                num_nodes += 1;
            }
        }
        let has_children = num_nodes > 1;
        let gd_type = gd_type_option
            .ok_or(anyhow!("{} no master node???", full_name))?
            .to_string();

        return Ok(Self {
            full_name,
            name,
            path,
            contents,
            script,
            has_children,
            gd_type,
        });
    }
}
