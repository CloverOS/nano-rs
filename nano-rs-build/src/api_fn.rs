use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use syn::{Attribute, FnArg, Item, ItemUse};
use syn::punctuated::Punctuated;
use syn::token::Comma;

use crate::api_doc::ApiFnDoc;
use crate::api_parse::{parse_fn_item, parse_fn_item_in_mod};

/// 构建API接口信息结构体
/// Build API interface information structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiFn<L, I, U, A> {
    /// api function name
    pub api_fn_name: String,
    /// layer function name
    pub layers_fn_name: Option<Vec<L>>,
    /// api input
    pub inputs: Option<I>,
    /// route path
    pub path: String,
    /// route group path
    pub path_group: String,
    /// http method
    pub method: String,
    /// is need auth
    pub public: bool,
    /// api function doc
    pub api_fn_doc: Option<ApiFnDoc>,
    /// use crate
    pub use_crate: Option<U>,
    /// attrs token steam
    pub attrs: Option<A>,
}

pub fn get_rs_files_fns(
    files: &mut Vec<PathBuf>,
) -> Result<HashMap<String, ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>>, Box<dyn Error>> {
    let mut fns = HashMap::new();
    for file in files {
        // 读入你的 Rust 源文件
        let src = fs::read_to_string(file.clone())?;
        // 解析Rust源代码为语法树
        eprintln!("parsing: {:?}", file.clone());
        let syntax_tree = syn::parse_file(&src)?;
        //先获取全部的use,防止有些文件没有进行rustfmt
        let mut item_uses: Vec<ItemUse> = vec![];
        for item in &syntax_tree.items {
            match item {
                Item::Use(item_use) => {
                    item_uses.push(item_use.clone());
                }
                _ => {}
            }
        }
        for item in &syntax_tree.items {
            match item {
                Item::Fn(item_fn) => {
                    if let Some(parsed) = parse_fn_item(item_fn, file.clone(), None)? {
                        let (fn_name, mut api_fn) = parsed;
                        eprintln!("add fn :{:?}", fn_name);
                        api_fn.use_crate = Some(item_uses.clone());
                        fns.insert(fn_name, api_fn);
                    }
                }
                Item::Mod(item_mod) => {
                    parse_fn_item_in_mod(
                        &mut fns,
                        item_mod,
                        item_mod.ident.to_string().as_str(),
                        file.clone(),
                    );
                }
                _ => {}
            };
        }
    }
    Ok(fns)
}
