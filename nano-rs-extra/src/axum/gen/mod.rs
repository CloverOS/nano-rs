#[cfg(feature = "utoipa_axum")]
use nano_rs_build::api_fn::ApiFn;
use proc_macro2::Ident;
#[cfg(feature = "utoipa_axum")]
use syn::punctuated::Punctuated;
#[cfg(feature = "utoipa_axum")]
use syn::token::Comma;
#[cfg(feature = "utoipa_axum")]
use syn::{Attribute, FnArg};
use syn::{ItemUse, UseGroup, UseName, UsePath, UseRename, UseTree};

pub mod gen_api_info;
pub mod gen_doc;
pub mod gen_route;

#[cfg(feature = "utoipa_axum")]
pub const UTOIPA_PATH: &str = "utoipapath";

pub trait AxumGen {
    fn match_use_tree(
        &self,
        tree: &UseTree,
        type_name: &str,
        parent_path: &mut Vec<Ident>,
    ) -> Option<String> {
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
                    Some(format!(
                        "{}::{}",
                        self.get_parent_path(&parent_path),
                        rename
                    ))
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

    fn get_full_crate_name(
        &self,
        type_name: String,
        item_use_vec: &Vec<ItemUse>,
    ) -> Option<String> {
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


/// trans openapi path to axum path, it will deprecated at next version(axum0.8)
#[cfg(feature = "utoipa_axum")]
pub fn trans_utoipa_to_axum(old: String) -> String {
    old.replace("{", ":").replace("}", "")
}

#[cfg(feature = "utoipa_axum")]
pub fn parse_utoipa_info(
    api_fn: &mut ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
    trans_fn: fn(String) -> String,
) {
    if let Some(attrs) = api_fn.clone().attrs {
        for attr in attrs.iter() {
            if attr.meta.path().segments.len() > 1 {
                let segments = attr.meta.path().clone().segments;
                let mut path_seg = "".to_string();
                for x in segments {
                    path_seg = format!("{}{}", path_seg, x.ident.to_string());
                }
                if path_seg == UTOIPA_PATH {
                    if let Ok(list) = attr.meta.require_list() {
                        let op_str = list.tokens.to_string();
                        let split: Vec<&str> = op_str.split(",").collect();
                        for x in split.iter() {
                            let kvs: Vec<&str> = x.split("=").collect();
                            if kvs.len() > 1 {
                                let key = kvs.first().unwrap_or(&"");
                                let value = kvs.last().unwrap_or(&"").replace("\"", "");
                                match key.trim() {
                                    "path" => {
                                        api_fn.path = trans_fn(value.trim().to_string());
                                    }
                                    "tag" => {
                                        if let Some(api_fn_doc) = &mut api_fn.api_fn_doc {
                                            api_fn_doc.api_group = value.trim().to_string();
                                        }
                                    }
                                    &_ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}