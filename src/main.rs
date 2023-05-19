use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use syn::__private::ToTokens;
use syn::{
    visit::Visit, File, Item, ItemConst, ItemEnum, ItemFn, ItemForeignMod, ItemMacro, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, Signature, Visibility,
};
use syn::{ItemUse, PathSegment, UseTree};

#[derive(Clone, Debug)]
struct Tree(HashMap<String, Option<Tree>>);

impl Tree {
    fn new() -> Self {
        Self(HashMap::new())
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn print_tree(tree: &Tree, prefix: String, f: &mut std::fmt::Formatter<'_>) {
            for (name, maybe_subtree) in &tree.0 {
                let new_prefix = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{}::{}", prefix, name)
                };

                if let Some(subtree) = maybe_subtree {
                    print_tree(&subtree, new_prefix, f);
                } else {
                    writeln!(f, "{}", new_prefix).unwrap();
                }
            }
        }

        print_tree(self, String::new(), f);

        Ok(())
    }
}

struct Visitor {
    current_dir: PathBuf,
    exports_tree: Tree,
    imports_tree: Tree,
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
            }) => Some((ident.to_string(), false)),
            Item::ForeignMod(ItemForeignMod { abi: _, items, .. }) => {
                unimplemented!()
            }
            Item::Macro(ItemMacro { mac, .. }) => {
                Some((mac.path.segments.last().unwrap().ident.to_string(), false))
            }
            Item::Mod(ItemMod { ident, .. }) => {
                let name = ident.to_string();
                let mod_dir = self.current_dir.join(&name);
                let mod_file = mod_dir.join("mod.rs");
                let alt_mod_file = self.current_dir.join(format!("{}.rs", &name));

                if mod_file.exists() {
                    self.visit_file(mod_file);
                } else if alt_mod_file.exists() {
                    self.visit_file(alt_mod_file);
                }

                Some((name, true))
            }
            Item::Use(use_item) => {
                self.visit_item_use(use_item);
                None
            }
            _ => return,
        };

        if let Some((name, is_module)) = item {
            self.exports_tree.0.entry(name).or_insert(if is_module {
                Some(Tree::new())
            } else {
                None
            });
        }

        syn::visit::visit_item(self, i);
    }

    fn visit_item_use(&mut self, i: &'ast ItemUse) {
        let tree = process_use_tree(&i.tree);
        self.imports_tree.0.extend(tree.0);
    }
}

impl Visitor {
    fn visit_file(&mut self, path: PathBuf) {
        let old_dir = self.current_dir.clone();
        let mut old_tree = self.exports_tree.clone();

        self.current_dir = path.parent().unwrap().into();
        self.exports_tree = Tree::new();

        let file_content = fs::read_to_string(&path).unwrap();
        let syntax_tree = syn::parse_file(&file_content).unwrap();

        syn::visit::visit_file(self, &syntax_tree);

        let module_name = path.file_stem().unwrap().to_str().unwrap().to_owned();

        old_tree
            .0
            .insert(module_name, Some(self.exports_tree.clone()));
        self.exports_tree = old_tree;
        self.current_dir = old_dir;
    }

    fn get_path_segment_name(&self, segment: &PathSegment) -> String {
        let segment_name = segment.ident.to_string();
        match &segment.arguments {
            syn::PathArguments::None => segment_name,
            syn::PathArguments::AngleBracketed(args) => {
                let mut generics = Vec::new();
                for arg in &args.args {
                    match arg {
                        syn::GenericArgument::Type(ty) => {
                            generics.push(ty.to_token_stream().to_string())
                        }
                        _ => (),
                    }
                }
                format!("{}<{}>", segment_name, generics.join(","))
            }
            syn::PathArguments::Parenthesized(_) => segment_name,
        }
    }
}

fn process_use_tree(tree: &UseTree) -> Tree {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = "/Users/dan.spencer/Sites/core-platform/anaplan-kube-operator/crs-controlplane/lib/crs_lib/src/lib.rs";
    let file_path = PathBuf::from(file);
    let mut visitor = Visitor {
        current_dir: file_path.parent().unwrap().into(),
        exports_tree: Tree::new(),
        imports_tree: Tree::new(),
    };
    visitor.visit_file(file_path);

    println!("File: {}", file);
    println!("Exports tree:");
    println!("{}", visitor.exports_tree);
    println!("Imports tree:");
    println!("{}", visitor.imports_tree);

    Ok(())
}
