use std::collections::HashMap;
use std::fmt::Display;

#[derive(Clone, Debug)]
pub struct Tree(pub HashMap<String, Option<Tree>>);

impl Tree {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn extend(&mut self, other: Tree) {
        for (key, value) in other.0 {
            self.0
                .entry(key)
                .and_modify(|e| {
                    if let Some(e) = e {
                        if let Some(value) = &value {
                            e.extend(value.clone());
                        }
                    } else {
                        *e = value.clone();
                    }
                })
                .or_insert(value);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_extends_empty_tree_with_empty_tree() {
        let mut tree1 = Tree::new();
        let tree2 = Tree::new();

        tree1.extend(tree2);

        assert_eq!(tree1.0.len(), 0);
    }

    #[test]
    fn it_extends_non_empty_tree_with_empty_tree() {
        let mut tree1 = Tree::new();
        tree1.0.insert("key1".to_string(), None);
        let tree2 = Tree::new();

        tree1.extend(tree2);

        assert_eq!(tree1.0.len(), 1);
        assert!(tree1.0.contains_key("key1"));
    }

    #[test]
    fn it_extends_empty_tree_with_non_empty_tree() {
        let mut tree1 = Tree::new();
        let mut tree2 = Tree::new();
        tree2.0.insert("key2".to_string(), None);

        tree1.extend(tree2);

        assert_eq!(tree1.0.len(), 1);
        assert!(tree1.0.contains_key("key2"));
    }

    #[test]
    fn it_merges_two_trees_with_different_keys() {
        let mut tree1 = Tree::new();
        tree1.0.insert("key1".to_string(), None);
        let mut tree2 = Tree::new();
        tree2.0.insert("key2".to_string(), None);

        tree1.extend(tree2);

        assert_eq!(tree1.0.len(), 2);
        assert!(tree1.0.contains_key("key1"));
        assert!(tree1.0.contains_key("key2"));
    }

    #[test]
    fn it_merges_two_trees_with_same_key_and_non_empty_values() {
        let mut tree1 = Tree::new();
        let mut sub_tree1 = Tree::new();
        sub_tree1.0.insert("sub_key1".to_string(), None);
        tree1.0.insert("key".to_string(), Some(sub_tree1));

        let mut tree2 = Tree::new();
        let mut sub_tree2 = Tree::new();
        sub_tree2.0.insert("sub_key2".to_string(), None);
        tree2.0.insert("key".to_string(), Some(sub_tree2));

        tree1.extend(tree2);

        assert_eq!(tree1.0.len(), 1);
        assert!(tree1.0.contains_key("key"));

        let merged_sub_tree = tree1.0.get("key").unwrap().as_ref().unwrap();
        assert_eq!(merged_sub_tree.0.len(), 2);
        assert!(merged_sub_tree.0.contains_key("sub_key1"));
        assert!(merged_sub_tree.0.contains_key("sub_key2"));
    }
}
