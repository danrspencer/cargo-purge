mod package;
mod tree;
mod visitor;
mod workspace;

use crate::tree::Tree;
use crate::visitor::Visitor;
use crate::workspace::Workspace;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut workspace = Workspace::new();
    workspace.load_from_file(
        "/Users/dan.spencer/Sites/core-platform/anaplan-kube-operator/crs-controlplane",
    )?;

    let (exports, import) = workspace.packages.into_iter().fold(
        (Tree::new(), Tree::new()),
        |(mut exports, mut imports), package| {
            let lib_file_path = PathBuf::from(format!(
                "{}/src/lib.rs",
                package.path.clone().to_str().unwrap()
            ));
            let main_file_path = PathBuf::from(format!(
                "{}/src/main.rs",
                package.path.clone().to_str().unwrap()
            ));

            let file_path = if Path::new(&lib_file_path).exists() {
                lib_file_path
            } else if Path::new(&main_file_path).exists() {
                main_file_path
            } else {
                panic!("Neither lib.rs nor main.rs found in package");
            };

            let mut visitor = Visitor::new(package.path);
            visitor.visit_file(file_path);

            exports.extend(visitor.exports_tree);
            imports.extend(visitor.imports_tree);
            (exports, imports)
        },
    );

    // println!("Exports tree:");
    // println!("{}", exports);
    // println!("Imports tree:");
    // println!("{}", import);

    let unused_exports = exports.filter_by(&import);

    println!("Unused exports:");
    println!("{}", unused_exports);

    Ok(())
}
