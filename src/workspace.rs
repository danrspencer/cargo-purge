use glob::glob;
use std::collections::HashSet;
use std::fs;
use std::io::Error;
use std::path::PathBuf;
use toml::Value;

pub struct Workspace {
    pub members: HashSet<PathBuf>,
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace {
            members: HashSet::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Error> {
        let cargo_toml = format!("{}/Cargo.toml", file_path);
        let contents = fs::read_to_string(cargo_toml)?;

        let value: Value = contents.parse().expect("Failed to parse toml file");

        let members = value
            .get("workspace")
            .and_then(|workspace| workspace.get("members"))
            .and_then(|members| members.as_array())
            .cloned()
            .unwrap_or_default();

        println!("members: {:?}", members);

        for item in members {
            if let Some(s) = item.as_str() {
                let path = format!("{}/{}", file_path, s);
                for entry in glob(&path).expect("Failed to read glob pattern") {
                    match entry {
                        Ok(path) => {
                            self.members.insert(path);
                        }
                        Err(e) => println!("{:?}", e),
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_members() {
        let mut workspace = Workspace::new();
        workspace.load_from_file("Cargo.toml").unwrap();

        for member in &workspace.members {
            println!("member: {}", member);
        }
    }
}
