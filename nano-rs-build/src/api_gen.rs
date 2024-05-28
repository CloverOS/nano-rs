use std::collections::HashMap;
use std::path::PathBuf;
use syn::{FnArg, ItemUse};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use crate::api_fn::ApiFn;

/// GenRoute trait
pub trait GenRoute {
    fn gen_route(&self, rs_files: Vec<PathBuf>, path_buf: PathBuf, api_fns: HashMap<String, ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>>>);

    fn get_routes_file_path(&self) -> &'static str {
        "src/routes.rs"
    }
}

/// GenDoc trait
pub trait GenDoc {
    fn gen_doc(&self, rs_files: Vec<PathBuf>, path_buf: PathBuf, api_fns: HashMap<String, ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>>>);

    fn get_doc_file_path(&self) -> &'static str {
        "src/doc.rs"
    }
}

/// GenApiInfo trait
pub trait GenApiInfo {
    fn gen_api_info(&self, path_buf: PathBuf, api_fns: HashMap<String, ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>>>);

    fn get_api_info_file_path(&self) -> &'static str {
        "src/api_info.rs"
    }
}