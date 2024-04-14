use std::collections::HashMap;
use std::path::PathBuf;

use syn::{FnArg, ItemUse};
use syn::punctuated::Punctuated;
use syn::token::Comma;

use crate::api_file::get_rs_files;
use crate::api_fn::{ApiFn, get_rs_files_fns};
use crate::api_gen::{GenDoc, GenRoute};

#[derive(Clone)]
pub struct NanoBuilder {
    api_fns: HashMap<String, ApiFn<String, Punctuated<FnArg, Comma>,Vec<ItemUse>>>,
    api_gen_path: PathBuf,
}


impl NanoBuilder {
    pub fn new(path: Option<PathBuf>) -> Self {
        let api_gen_path;
        if let Some(path_buf) = path {
            api_gen_path = path_buf;
        } else {
            api_gen_path = std::env::current_dir().expect("get current dir error");
        }
        let mut rs_files = Vec::new();
        get_rs_files(&mut rs_files, api_gen_path.as_path()).expect("get rs files error");
        let api_fns = get_rs_files_fns(&mut rs_files).expect("get rs files fns error");
        eprintln!("get {} api things", api_fns.len());
        NanoBuilder {
            api_fns,
            api_gen_path,
        }
    }

    pub fn gen_api_route(&mut self, gen_route: impl GenRoute) -> &mut Self {
        gen_route.gen_route(self.clone().api_gen_path, self.api_fns.clone());
        self
    }

    pub fn gen_api_doc(&mut self, gen_doc: impl GenDoc) -> &mut Self {
        gen_doc.gen_doc(self.clone().api_gen_path, self.api_fns.clone());
        self
    }
}

