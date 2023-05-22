mod package;
mod tree;
mod visitor;
mod workspace;

use crate::tree::Tree;
use crate::visitor::Visitor;
use cargo::core::Workspace;
use cargo::util::command_prelude::*;
use cargo::CliResult;
use cargo::Config;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn main() {
    let current_path = std::env::current_dir().unwrap();

    let unused_exports = find_unused_exports(current_path);

    println!("Unused exports:");
    println!("{}", unused_exports);
}

fn find_unused_exports(workspace_path: PathBuf) -> Tree<String> {
    let manifest_path = Path::new(&workspace_path).join("Cargo.toml");

    let config = Config::default().unwrap();
    let workspace = Workspace::new(&manifest_path, &config).expect("Failed to load workspace");

    let (exports, imports) = workspace.members().into_iter().fold(
        (Tree::new(), Tree::new()),
        |(mut exports, mut imports), package| {
            // Todo - can we figure out the entry point from the Package struct?
            let lib_file_path = Path::new(&package.root()).join("src").join("lib.rs");
            let main_file_path = Path::new(&package.root()).join("src").join("main.rs");

            let file_path = if Path::new(&lib_file_path).exists() {
                lib_file_path
            } else if Path::new(&main_file_path).exists() {
                main_file_path
            } else {
                panic!("Neither lib.rs nor main.rs found in package");
            };

            let mut visitor = Visitor::new(package.root().into());
            visitor.visit_file(file_path);

            exports.extend(visitor.exports_tree);
            imports.extend(visitor.imports_tree);
            (exports, imports)
        },
    );

    let unused_exports = exports.filter_by(&imports);

    unused_exports
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn it_correctly_finds_unused_exports() {
        let unused_exports = find_unused_exports("test_workspaces/workspace_1");
        let unused_exports = serde_json::to_value(unused_exports).unwrap();

        assert_eq!(
            unused_exports,
            json!({ "package_1": { "public_hello_3": null }})
        )
    }
}
