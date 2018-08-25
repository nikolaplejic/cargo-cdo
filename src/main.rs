extern crate toml;

use std::fs::File;
use std::io::{Error, ErrorKind};
use std::io::prelude::*;
use std::collections::{HashMap, BTreeMap};

use toml::Value;

fn load_toml(filename: &str) -> Value {
    let mut toml_file = File::open(filename).unwrap();
    let mut toml_str = String::new();

    toml_file.read_to_string(&mut toml_str).unwrap();

    toml_str.parse::<Value>().unwrap()
}

fn main() -> std::io::Result<()> {
    let workspace_toml = load_toml("Cargo.toml");
    let workspace_toml = workspace_toml.as_table().unwrap();

    if !workspace_toml.contains_key("workspace") {
        return Err(Error::new(ErrorKind::Other, "Not in a workspace!"));
    }

    let workspace = workspace_toml.get("workspace").unwrap();
    let members = match workspace.get("members") {
        Some(members) => members.as_array().unwrap(),
        None          => panic!("No members in the workspace, exiting...")
    };

    let mut member_dependencies = HashMap::new();

    for member in members {
        let member_str = member.as_str().unwrap();

        let crate_toml = load_toml(
            &format!("{}/Cargo.toml", member_str)
        );
        let crate_toml = crate_toml.as_table().unwrap();

        let empty_deps = BTreeMap::new();
        let deps = match crate_toml.get("dependencies") {
            Some(deps) => deps.as_table().unwrap(),
            None       => &empty_deps
        };

        let mut deps_v = Vec::new();

        for key in deps.keys() {
            let version = deps.get(key).unwrap();

            if version.is_str() {
                deps_v.push((key.to_owned(), version.as_str().unwrap().to_owned()));
            }
        }

        member_dependencies.insert(member_str, deps_v);
    }

    let mut ws_deps = HashMap::new();

    for (member, deps) in &member_dependencies {
        for (dep, version) in deps {
            ws_deps.entry(dep.to_owned()).or_insert(Vec::new());
            ws_deps.get_mut(&dep.to_owned()).unwrap().push((member.to_owned(), version.to_owned()));
        }
    }

    for (dep, members) in &ws_deps {
        if members.len() > 1 {
            println!("{} - {:?}", dep, members);
        }
    }

    Ok(())
}
