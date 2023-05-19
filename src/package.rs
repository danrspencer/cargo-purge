use std::hash::{Hash, Hasher};
use std::{collections::HashSet, path::PathBuf};

#[derive(Debug, Eq, PartialEq)]
pub struct Package {
    name: String,
    path: PathBuf,
    dependencies: HashSet<String>,
}

impl Package {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .components()
            .last()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_owned();

        let file_path = path.clone();

        Self {
            name,
            path,
            dependencies: load_from_file(file_path.to_str().unwrap()).unwrap_or_default(),
        }
    }
}

impl Hash for Package {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

fn load_from_file(file_path: &str) -> Result<HashSet<String>, std::io::Error> {
    let cargo_toml = format!("{}/Cargo.toml", file_path);
    let contents = std::fs::read_to_string(cargo_toml)?;

    let value: toml::Value = contents.parse().expect("Failed to parse toml file");

    let dependencies = value
        .get("dependencies")
        .and_then(|dependencies| dependencies.as_table())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(dep, val)| {
            // we only care about depenencies with a path
            val.as_table()?.get("path").map(|_| dep)
        })
        .collect();

    Ok(dependencies)
}
