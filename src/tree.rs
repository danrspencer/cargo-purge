use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Tree<T: Clone + Eq + Hash + PartialEq>(pub HashMap<T, Option<Tree<T>>>);

impl<T: Clone + Eq + Hash + PartialEq> Tree<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn extend(&mut self, other: Tree<T>) {
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

    pub fn insert(&mut self, key: T, value: Option<Tree<T>>) {
        self.0.insert(key, value);
    }

    pub fn filter_by(&self, other: &Tree<T>) -> Tree<T> {
        let mut filtered_nodes = HashMap::new();

        for (key, value) in &self.0 {
            if let Some(Some(other_tree)) = other.0.get(key) {
                if let Some(tree) = value {
                    let filtered_tree = tree.filter_by(other_tree);
                    if !filtered_tree.0.is_empty() {
                        filtered_nodes.insert(key.clone(), Some(filtered_tree));
                    }
                } else {
                    filtered_nodes.insert(key.clone(), None);
                }
            } else {
                filtered_nodes.insert(key.clone(), value.clone());
            }
        }

        Tree(filtered_nodes)
    }
}

impl<T: Clone + Display + Eq + Hash + PartialEq> Display for Tree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn print_tree<T: Clone + Display + Eq + Hash + PartialEq>(
            tree: &Tree<T>,
            prefix: String,
            f: &mut std::fmt::Formatter<'_>,
        ) {
            for (name, maybe_subtree) in &tree.0 {
                let new_prefix = if prefix.is_empty() {
                    name.clone().to_string()
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
    use serde_json::json;

    #[test]
    fn it_extends_empty_tree_with_empty_tree() {
        let mut tree1: Tree<String> = Tree::new();
        let tree2: Tree<String> = Tree::new();

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

    #[test]
    fn it_serializes_as_expected() {
        let mut tree = Tree::new();
        let mut sub_tree = Tree::new();
        sub_tree.0.insert("sub_key".to_string(), None);
        tree.0.insert("key".to_string(), Some(sub_tree));

        let serialized = serde_json::to_value(&tree).unwrap();

        assert_eq!(serialized, json!({"key":{"sub_key":null}}));
    }
}
