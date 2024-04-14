use std::collections::HashMap;
use std::path::PathBuf;
use syn::{FnArg, ItemUse};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use crate::api_fn::ApiFn;

pub trait GenRoute {
    fn gen_route(&self, path_buf: PathBuf, api_fns: HashMap<String, ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>>>);

    fn get_routes_file_path(&self) -> &'static str {
        "src/routes.rs"
    }
}

pub trait GenDoc {
    fn gen_doc(&self, path_buf: PathBuf, api_fns: HashMap<String, ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>>>);
}