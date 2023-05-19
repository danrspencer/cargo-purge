mod package;
mod tree;
mod visitor;
mod workspace;

use crate::visitor::Visitor;
use crate::workspace::Workspace;
use std::collections::HashSet;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = "/Users/dan.spencer/Sites/core-platform/anaplan-kube-operator/crs-controlplane/lib/crs_lib/src/lib.rs";
    // let file_path = PathBuf::from(file);
    // let mut visitor = Visitor::new(file_path.parent().unwrap().into());
    // visitor.visit_file(file_path);

    // println!("File: {}", file);
    // println!("Exports tree:");
    // println!("{}", visitor.exports_tree);
    // println!("Imports tree:");
    // println!("{}", visitor.imports_tree);

    let mut workspace = Workspace::new();
    workspace.load_from_file(
        "/Users/dan.spencer/Sites/core-platform/anaplan-kube-operator/crs-controlplane",
    )?;
    println!("Workspace members: {:?}", workspace.packages);

    Ok(())
}
