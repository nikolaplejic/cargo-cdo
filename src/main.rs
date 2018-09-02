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

fn dependency_map(cargo_files: &Vec<String>) -> DependencyMap {
    let mut dm = DependencyMap::new();

    for member in cargo_files {
        let crate_str = load_toml(&member)
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

    return dm;
}

fn detect_dupes(dm: DependencyMap) -> DependencyMap {
    dm.into_iter()
        .filter(|(_, v)| { v.len() > 1 })
        .collect()
}

fn normalize_deps(vers: &Vec<DependencyEntry>) -> Vec<String> {
    let mut version_nos : Vec<String> = vers.iter()
        .map(|ref de| de.version.to_string())
        .collect();

    version_nos.sort();
    version_nos.dedup();

    return version_nos;
}

fn main() -> std::io::Result<()> {
    let toml_str = load_toml("Cargo.toml").expect("No Cargo.toml file found.");

    let tl_workspace : CargoWorkspace = match toml::from_str(&toml_str) {
        Ok(ws) => ws,
        _      => panic!("Malformed Cargo.toml -- are you in a workspace?")
    };
    let workspace = tl_workspace.workspace;

    let members = workspace.members
        .into_iter()
        .map(|m| format!("{}/Cargo.toml", m))
        .collect();

    let dm = dependency_map(&members);
    let repeating_deps = detect_dupes(dm);

    for (dep, vers) in &repeating_deps {
        let version_nos = normalize_deps(&vers);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_parsing() {
        let workspace_str = load_toml("fixture/workspace.toml").unwrap();
        let tl_workspace : CargoWorkspace = toml::from_str(&workspace_str).unwrap();

        assert_eq!(4, tl_workspace.workspace.members.len());
        assert_eq!("awesome_project", tl_workspace.workspace.members.first().unwrap());
    }

    #[test]
    fn test_dependency_map() {
        let tomls = vec![
            "fixture/crate1.toml".to_string(),
            "fixture/crate2.toml".to_string(),
        ];
        let dm = dependency_map(&tomls);

        let serde = dm.get("serde").unwrap();
        assert_eq!(2, serde.len());

        let internal_project = dm.get("internal_project").unwrap();
        assert_eq!(1, internal_project.len());
    }

    #[test]
    fn test_dupe_detection() {
        let tomls = vec![
            "fixture/crate1.toml".to_string(),
            "fixture/crate2.toml".to_string(),
        ];
        let dm = dependency_map(&tomls);

        let repeating_deps = detect_dupes(dm);

        assert!(repeating_deps.contains_key("serde"));
        assert!(repeating_deps.contains_key("toml"));
        assert!(!repeating_deps.contains_key("unknown_project"));
    }
}
