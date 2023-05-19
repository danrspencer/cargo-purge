use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use syn::ItemUse;
use syn::{
    visit::Visit, File, Item, ItemConst, ItemEnum, ItemFn, ItemForeignMod, ItemMacro, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, Signature, Visibility,
};

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

enum BasicItem {
    Export { name: String, is_module: bool },
    Import { name: String },
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
            }) => BasicItem::Export {
                name: ident.to_string(),
                is_module: false,
            },
            Item::ForeignMod(ItemForeignMod { abi: _, items, .. }) => {
                unimplemented!()
            }
            Item::Macro(ItemMacro { mac, .. }) => BasicItem::Export {
                name: mac.path.segments.last().unwrap().ident.to_string(),
                is_module: false,
            },
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

                BasicItem::Export {
                    name,
                    is_module: true,
                }
            }
            // Item::Use(use_item) => self.visit_item_use(use_item),
            _ => return,
        };

        match item {
            BasicItem::Export { name, is_module } => {
                self.exports_tree.0.entry(name).or_insert(if is_module {
                    Some(Tree::new())
                } else {
                    None
                });
            }
            _ => unimplemented!(),
        }

        syn::visit::visit_item(self, i);
    }

    // fn visit_item_use(&mut self, i: &ItemUse) {
    //     let mut path_parts = Vec::new();

    //     for segment in &i.tree {
    //         path_parts.push(self.get_path_segment_name(segment));
    //     }

    //     let path_str = path_parts.join("::");

    //     self.imports_tree.insert(path_str, None);
    // }
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

    // fn get_path_segment_name(&self, segment: &PathSegment) -> String {
    //     let segment_name = segment.ident.to_string();
    //     match &segment.arguments {
    //         syn::PathArguments::None => segment_name,
    //         syn::PathArguments::AngleBracketed(args) => {
    //             let mut generics = Vec::new();
    //             for arg in &args.args {
    //                 match arg {
    //                     syn::GenericArgument::Type(ty) => {
    //                         generics.push(ty.to_token_stream().to_string())
    //                     }
    //                     _ => (),
    //                 }
    //             }
    //             format!("{}<{}>", segment_name, generics.join(","))
    //         }
    //         syn::PathArguments::Parenthesized(_) => segment_name,
    //     }
    // }
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
    println!("{}", visitor.exports_tree);

    Ok(())
}
