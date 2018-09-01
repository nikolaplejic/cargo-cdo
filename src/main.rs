#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;

use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct CargoWorkspace {
    workspace: Workspace,
}

#[derive(Deserialize, Debug)]
struct Workspace {
    members: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct CargoCrate {
    package: Package,
    dependencies: HashMap<String, toml::Value>,
}

#[derive(Deserialize, Debug)]
struct Package {
    name: String,
    version: String,
}

#[derive(Debug)]
struct DependencyEntry {
    project: String,
    version: String,
}

type DependencyMap = HashMap<String, Vec<DependencyEntry>>;

fn load_toml(filename: &str) -> Result<String, std::io::Error> {
    let mut toml_file = File::open(filename)?;
    let mut toml_str = String::new();

    toml_file.read_to_string(&mut toml_str)?;

    Ok(toml_str.to_string())
}

fn main() -> std::io::Result<()> {
    let mut dm = DependencyMap::new();

    let toml_str = load_toml("Cargo.toml").expect("No Cargo.toml file found.");

    let tl_workspace : CargoWorkspace = match toml::from_str(&toml_str) {
        Ok(ws) => ws,
        _      => panic!("Malformed Cargo.toml -- are you in a workspace?")
    };
    let workspace = tl_workspace.workspace;

    for member in workspace.members {
        let crate_str = load_toml(&format!("{}/Cargo.toml", member))
            .expect("No Cargo.toml file found.");

        let member_toml : CargoCrate = match toml::from_str(&crate_str) {
            Ok(member) => member,
            _          => panic!("Malformed Cargo.toml")
        };

        for (name, version) in member_toml.dependencies {
            let version = match version {
                toml::Value::String(ver) => ver,
                toml::Value::Table(tbl) => {
                    match tbl.get("version") {
                        Some(toml::Value::String(ver)) => ver.to_string(),
                        _ => "".to_string()
                    }
                }
                _ => "".to_string()
            };

            let entry = DependencyEntry {
                project: member.to_string(),
                version: version.to_string()
            };

            dm.entry(name)
                .or_insert(vec![])
                .push(entry);
        }
    }

    let repeating_deps : DependencyMap =
        dm.into_iter().filter(|(_, v)| { v.len() > 1 }).collect();

    for (dep, vers) in &repeating_deps {
        let mut version_nos : Vec<String> = vers.iter()
            .map(|ref de| de.version.to_string())
            .collect();

        version_nos.sort();
        version_nos.dedup();

        if version_nos.len() > 1 {
            println!("Found duplicate versions for dependency {}", dep);

            for version in vers {
                println!("{} - {}", version.project, version.version);
            }

            println!("");
        }
    }

    Ok(())
}
