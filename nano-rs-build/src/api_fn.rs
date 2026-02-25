use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, FnArg, Item, ItemUse};

use crate::api_doc::ApiFnDoc;
use crate::api_parse::{
    CrateContext, gen_file_crate_module_path, gen_fn_full_crate_path, parse_fn_item,
    parse_fn_item_in_mod, parse_mod_tag_attr, resolve_crate_context,
};

struct ParsedRsFile {
    path: PathBuf,
    crate_ctx: CrateContext,
    syntax_tree: syn::File,
}

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
    /// function return type tokens (for auto doc inference)
    pub output: Option<String>,
    /// whether `#[utoipa::path(..)]` exists on handler
    pub has_utoipa_path: bool,
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
    let mut module_tag_index: HashMap<String, String> = HashMap::new();
    let mut parsed_files: Vec<ParsedRsFile> = Vec::new();

    for file in files.iter() {
        let src = fs::read_to_string(file)?;
        let syntax_tree = syn::parse_file(&src)?;
        let crate_ctx = resolve_crate_context(file.as_path(), base_path, &mut crate_cache);
        parsed_files.push(ParsedRsFile {
            path: file.clone(),
            crate_ctx,
            syntax_tree,
        });
    }

    for parsed in parsed_files.iter() {
        for item in &parsed.syntax_tree.items {
            if let Item::Mod(item_mod) = item {
                if let Some(tag) = parse_mod_tag_attr(&item_mod.attrs) {
                    let module_path = gen_fn_full_crate_path(
                        &parsed.crate_ctx,
                        parsed.path.as_path(),
                        item_mod.ident.to_string(),
                        None,
                    );
                    module_tag_index.insert(module_path, tag);
                }
            }
        }
    }

    for parsed_file in parsed_files {
        let file_module_path =
            gen_file_crate_module_path(&parsed_file.crate_ctx, parsed_file.path.as_path());
        let file_default_tag = module_tag_index.get(&file_module_path).cloned();
        //先获取全部的use,防止有些文件没有进行rustfmt
        let mut item_uses: Vec<ItemUse> = vec![];
        for item in &parsed_file.syntax_tree.items {
            match item {
                Item::Use(item_use) => {
                    item_uses.push(item_use.clone());
                }
                _ => {}
            }
        }
        for item in &parsed_file.syntax_tree.items {
            match item {
                Item::Fn(item_fn) => {
                    if let Some(parsed_fn) = parse_fn_item(
                        item_fn,
                        parsed_file.path.clone(),
                        None,
                        &parsed_file.crate_ctx,
                        file_default_tag.as_deref(),
                    )? {
                        let (fn_name, mut api_fn) = parsed_fn;
                        eprintln!("add fn :{:?}", fn_name);
                        api_fn.use_crate = Some(item_uses.clone());
                        api_fn.crate_prefix = parsed_file.crate_ctx.code_prefix.clone();
                        api_fn.crate_name = Some(parsed_file.crate_ctx.package_name.clone());
                        fns.insert(fn_name, api_fn);
                    }
                }
                Item::Mod(item_mod) => {
                    let top_tag = parse_mod_tag_attr(&item_mod.attrs).or(file_default_tag.clone());
                    parse_fn_item_in_mod(
                        &mut fns,
                        item_mod,
                        item_mod.ident.to_string().as_str(),
                        parsed_file.path.clone(),
                        &parsed_file.crate_ctx,
                        top_tag.as_deref(),
                    )?;
                }
                _ => {}
            };
        }
    }
    Ok(fns)
}
