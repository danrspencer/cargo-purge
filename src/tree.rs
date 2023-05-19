use std::collections::HashMap;
use std::fmt::Display;

#[derive(Clone, Debug)]
pub struct Tree(pub HashMap<String, Option<Tree>>);

impl Tree {
    pub fn new() -> Self {
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
