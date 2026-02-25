use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, FnArg, Item, ItemUse};

use crate::api_doc::ApiFnDoc;
use crate::api_parse::{CrateContext, parse_fn_item, parse_fn_item_in_mod, resolve_crate_context};

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
    /// crate prefix used when referencing this handler (e.g. `crate` or external crate name)
    pub crate_prefix: String,
    /// actual crate package name
    pub crate_name: Option<String>,
}

pub fn get_rs_files_fns(
    files: &mut Vec<PathBuf>,
    base_path: &Path,
) -> Result<
    HashMap<String, ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>>,
    Box<dyn Error>,
> {
    let mut fns = HashMap::new();
    let mut crate_cache: HashMap<PathBuf, CrateContext> = HashMap::new();
    for file in files {
        // 读入你的 Rust 源文件
        let src = fs::read_to_string(file.clone())?;
        // 解析Rust源代码为语法树
        eprintln!("parsing: {:?}", file.clone());
        let syntax_tree = syn::parse_file(&src)?;
        let crate_ctx = resolve_crate_context(file.as_path(), base_path, &mut crate_cache);
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
                    if let Some(parsed) = parse_fn_item(item_fn, file.clone(), None, &crate_ctx)? {
                        let (fn_name, mut api_fn) = parsed;
                        eprintln!("add fn :{:?}", fn_name);
                        api_fn.use_crate = Some(item_uses.clone());
                        api_fn.crate_prefix = crate_ctx.code_prefix.clone();
                        api_fn.crate_name = Some(crate_ctx.package_name.clone());
                        fns.insert(fn_name, api_fn);
                    }
                }
                Item::Mod(item_mod) => {
                    parse_fn_item_in_mod(
                        &mut fns,
                        item_mod,
                        item_mod.ident.to_string().as_str(),
                        file.clone(),
                        &crate_ctx,
                    );
                }
                _ => {}
            };
        }
    }
    Ok(fns)
}
