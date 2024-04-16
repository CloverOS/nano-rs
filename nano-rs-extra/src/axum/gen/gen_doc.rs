use std::collections::HashMap;
use std::path::PathBuf;

use syn::{FnArg, ItemUse};
use syn::punctuated::Punctuated;
use syn::token::Comma;

use nano_rs_build::api_fn::ApiFn;
use nano_rs_build::api_gen::GenDoc;

pub struct AxumGenDoc{}
impl GenDoc for AxumGenDoc{
    fn gen_doc(&self, _path_buf: PathBuf, _api_fns: HashMap<String, ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>>>) {
        //todo
    }
}