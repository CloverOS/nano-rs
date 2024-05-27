use proc_macro2::Ident;
use syn::{ItemUse, UseGroup, UseName, UsePath, UseRename, UseTree};

pub mod gen_route;
pub mod gen_doc;
pub mod gen_api_info;

pub trait AxumGen {
    fn match_use_tree(&self, tree: &UseTree, type_name: &str, parent_path: &mut Vec<Ident>) -> Option<String> {
        match tree {
            UseTree::Path(UsePath { ident, tree, .. }) => {
                parent_path.push(ident.clone());
                self.match_use_tree(tree, type_name, parent_path)
            }
            UseTree::Name(UseName { ident }) => {
                if ident == type_name {
                    Some(format!("{}::{}", self.get_parent_path(&parent_path), ident))
                } else {
                    None
                }
            }
            UseTree::Rename(UseRename { ident, rename, .. }) => {
                let ident_str = format!("{}", ident);
                if &ident_str == type_name || rename == type_name {
                    Some(format!("{}::{}", self.get_parent_path(&parent_path), rename))
                } else {
                    None
                }
            }
            UseTree::Group(UseGroup { items, .. }) => {
                for item in items {
                    if let Some(found) = self.match_use_tree(item, type_name, parent_path) {
                        return Some(found);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn get_full_crate_name(&self, type_name: String, item_use_vec: &Vec<ItemUse>) -> Option<String> {
        for item_use in item_use_vec.iter() {
            let result = self.match_use_tree(&item_use.tree, &type_name, &mut vec![]);
            if result.is_some() {
                return result;
            }
        }
        None
    }

    fn get_parent_path(&self, path: &Vec<Ident>) -> String {
        let mut path_string = String::new();
        for (i, segment) in path.iter().enumerate() {
            path_string.push_str(&segment.to_string());
            if i < path.len() - 1 {
                path_string.push_str("::");
            }
        }
        path_string
    }
}