use crate::tree::Tree;
use std::fs;
use std::path::PathBuf;
use syn::{
    visit::Visit, Item, ItemConst, ItemEnum, ItemFn, ItemForeignMod, ItemMacro, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, Signature, Visibility,
};
use syn::{ItemUse, UseTree};

pub struct Visitor {
    pub current_dir: PathBuf,
    pub exports_tree: Tree<String>,
    pub imports_tree: Tree<String>,
}

impl Visitor {
    pub fn new(path: PathBuf) -> Self {
        Self {
            current_dir: path,
            exports_tree: Tree::new(),
            imports_tree: Tree::new(),
        }
    }

    pub fn visit_file(&mut self, path: PathBuf) {
        let old_dir = self.current_dir.clone();
        let mut old_tree = self.exports_tree.clone();

        self.current_dir = path.parent().unwrap().into();
        self.exports_tree = Tree::new();

        let file_content = fs::read_to_string(&path).unwrap();
        let syntax_tree = syn::parse_file(&file_content).unwrap();

        syn::visit::visit_file(self, &syntax_tree);

        // todo - make this not awful
        let module_name = if path.file_name().unwrap() == "lib.rs" {
            path.parent()
                .unwrap()
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        } else if path.file_name().unwrap() == "mod.rs" {
            path.parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        } else {
            path.file_stem().unwrap().to_str().unwrap().to_owned()
        };

        old_tree
            .0
            .insert(module_name, Some(self.exports_tree.clone()));

        self.exports_tree = old_tree;
        self.current_dir = old_dir;
    }
}

fn process_use_tree(tree: &UseTree) -> Tree<String> {
    match tree {
        UseTree::Path(use_path) => {
            let mut result = Tree::new();
            let subtree = process_use_tree(&*use_path.tree);
            result.0.insert(use_path.ident.to_string(), Some(subtree));
            result
        }
        UseTree::Name(use_name) => {
            let mut result = Tree::new();
            result.0.insert(use_name.ident.to_string(), None);
            result
        }
        UseTree::Rename(use_rename) => {
            let mut result = Tree::new();
            result.0.insert(use_rename.rename.to_string(), None);
            result
        }
        UseTree::Glob(_) => {
            let mut result = Tree::new();
            result.0.insert("*".to_string(), None);
            result
        }
        UseTree::Group(use_group) => {
            let mut result = Tree::new();
            for tree in &use_group.items {
                let subtree = process_use_tree(tree);
                result.0.extend(subtree.0);
            }
            result
        }
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_item(&mut self, i: &'ast Item) {
        let item = match i {
            Item::Struct(ItemStruct {
                vis: Visibility::Public(_),
                ident,
                ..
            })
            | Item::Enum(ItemEnum {
                vis: Visibility::Public(_),
                ident,
                ..
            })
            | Item::Fn(ItemFn {
                vis: Visibility::Public(_),
                sig: Signature { ident, .. },
                ..
            })
            | Item::Const(ItemConst {
                vis: Visibility::Public(_),
                ident,
                ..
            })
            | Item::Static(ItemStatic {
                vis: Visibility::Public(_),
                ident,
                ..
            })
            | Item::Trait(ItemTrait {
                vis: Visibility::Public(_),
                ident,
                ..
            })
            | Item::TraitAlias(ItemTraitAlias {
                vis: Visibility::Public(_),
                ident,
                ..
            })
            | Item::Type(ItemType {
                vis: Visibility::Public(_),
                ident,
                ..
            }) => Some(ident.to_string()),
            Item::ForeignMod(ItemForeignMod {
                abi: _, items: _, ..
            }) => {
                unimplemented!()
            }
            Item::Macro(ItemMacro { mac, .. }) => {
                Some(mac.path.segments.last().unwrap().ident.to_string())
            }
            Item::Mod(ItemMod {
                vis,
                ident,
                content,
                ..
            }) => {
                let name = ident.to_string();

                if content.is_none() {
                    let mod_dir = self.current_dir.join(&name);
                    let mod_file = mod_dir.join("mod.rs");
                    let alt_mod_file = self.current_dir.join(format!("{}.rs", &name));

                    let mut visitor = Visitor::new(mod_dir);

                    if mod_file.exists() {
                        visitor.visit_file(mod_file);
                    } else if alt_mod_file.exists() {
                        visitor.visit_file(alt_mod_file);
                    }

                    if matches!(vis, Visibility::Public(_)) {
                        self.exports_tree.extend(visitor.exports_tree);
                    }

                    self.imports_tree.extend(visitor.imports_tree);
                }

                if matches!(vis, Visibility::Public(_)) {
                    Some(name)
                } else {
                    None
                }
            }
            Item::Use(use_item) => {
                self.visit_item_use(use_item);
                None
            }
            _ => return,
        };

        if let Some(name) = item {
            self.exports_tree.0.entry(name).or_insert(None);
        }

        syn::visit::visit_item(self, i);
    }

    fn visit_item_use(&mut self, i: &'ast ItemUse) {
        let tree = process_use_tree(&i.tree);
        self.imports_tree.extend(tree);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn it_correctly_gets_public_exports() {
        let path = PathBuf::from("test_workspaces/workspace_1/lib/package_1/src/lib.rs");
        let mut visitor = Visitor::new(path.parent().unwrap().into());
        visitor.visit_file(PathBuf::from(path));

        let exports = serde_json::to_value(visitor.exports_tree).unwrap();

        assert_eq!(
            exports,
            json!({
                "package_1": {
                    "public_hello_1": null,
                    "public_hello_2": null,
                    "public_hello_3": null,
                    "public_module": {
                        "public_hello": null,
                        "public": {
                            "public_hello": null,
                        },
                    }
                },
            })
        )
    }

    #[test]
    fn it_correctly_gets_all_imports() {
        let path = PathBuf::from("test_workspaces/workspace_1/lib/package_2/src/lib.rs");

        let mut visitor = Visitor::new(path.parent().unwrap().into());
        visitor.visit_file(PathBuf::from(path));

        let imports = serde_json::to_value(visitor.imports_tree).unwrap();

        assert_eq!(
            imports,
            json!({
                "package_1": {
                    "public_hello_1": null,
                    "public_hello_2": null,
                    "public_module": {
                        "public_hello": null,
                        "public": {
                            "public_hello": null,
                        },
                    },
                },
                "super": {
                    "*": null
                }
            })
        )
    }
}
