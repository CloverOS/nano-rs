use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use quote::__private::TokenStream;
use quote::quote;
use syn::{Attribute, FnArg, ItemUse, parse_quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;

use nano_rs_build::api_fn::ApiFn;
use nano_rs_build::api_gen::GenApiInfo;

pub struct AxumGenApiInfo {}

impl GenApiInfo for AxumGenApiInfo {
    fn gen_api_info(&self, path_buf: PathBuf, api_fns: HashMap<String, ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>>) {
        eprintln!("gen_api_info in {:?}", path_buf);
        let api_info = path_buf.join(self.get_api_info_file_path());
        if !api_info.exists() {
            fs::write(api_info.as_path(), "").expect("create api info files error");
        }
        let api_info_struct_code = quote!(
            #[allow(dead_code)]
            #[derive(Debug,Clone)]
            pub struct ApiInfo {
                pub method: String,
                pub path: String,
                pub base_path: String,
                pub handler_fun: String,
                pub summary: String,
                pub public: bool,
                pub group_name: String,
            }
        );
        let mut api_info_vec: Vec<TokenStream> = vec![];
        let mut keys: Vec<_> = api_fns.keys().collect();
        keys.sort();
        for k in keys {
            let api_fn = api_fns.get(k).unwrap().clone();
            let method = api_fn.method;
            let path = api_fn.path;
            let base_path = api_fn.path_group;
            let handler_fun = api_fn.api_fn_name;
            let mut summary = "".to_string();
            let mut group_name = "".to_string();
            if let Some(api_doc) = api_fn.api_fn_doc {
                summary = api_doc.api;
                group_name = api_doc.api_group;
            }
            let public = api_fn.public;
            api_info_vec.push(quote! {
                ApiInfo {
                    method: #method.to_string(),
                    path: #path.to_string(),
                    base_path: #base_path.to_string(),
                    handler_fun: #handler_fun.to_string(),
                    summary: #summary.to_string(),
                    public: #public,
                    group_name: #group_name.to_string(),
                }
            });
        }
        let api_info_code: TokenStream = parse_quote!(
            /// Code generated by nano-rs. DO NOT EDIT.
            #[allow(dead_code)]
            pub fn get_api_info() -> Vec<ApiInfo> {
                vec![#(#api_info_vec),*]
            }

            #api_info_struct_code
        );

        let syntax_tree = syn::parse_file(api_info_code.to_string().as_str()).unwrap();
        let formatted = prettyplease::unparse(&syntax_tree);
        fs::write(api_info.as_path(), formatted).expect("create file failed");
    }
}

impl AxumGenApiInfo {
    pub fn new() -> Self {
        AxumGenApiInfo {}
    }
}