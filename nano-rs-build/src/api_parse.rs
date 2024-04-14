use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use syn::{Attribute, FnArg, Item, ItemFn, ItemMod, ItemUse};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use crate::api_doc::ApiFnDoc;
use crate::api_fn::ApiFn;
use crate::api_syn::ApiMacroInfo;

pub fn parse_api_info(item_fn: &ItemFn, attr: &Attribute, method: &str) -> Result<ApiFn<String, Punctuated<FnArg, Comma>,Vec<ItemUse>>, Box<dyn Error>> {
    let api_macro_info = attr.parse_args::<ApiMacroInfo>()?;
    let open_token = if let Some(open) = api_macro_info.open_token {
        open.value_token.value
    } else {
        false
    };
    let mut layers = Vec::new();
    if let Some(layers_token) = api_macro_info.layers_token {
        layers_token.value_token.iter().for_each(|layer| {
            let path_str = layer.value(); // 使用 value 方法获取 LitStr 中的字符串
            layers.push(path_str);
        });
    }

    let api_fn = ApiFn {
        api_fn_name: item_fn.sig.ident.to_string(),
        layers_fn_name: if layers.is_empty() {
            None
        } else {
            Some(layers)
        },
        inputs: Some(item_fn.sig.inputs.clone()),
        path: api_macro_info.path_token.value_token.value(),
        path_group: if let Some(path_group) = api_macro_info.path_group_token {
            path_group.value_token.value()
        } else {
            "".to_string()
        },
        method: method.to_string(),
        public: open_token,
        api_fn_doc: Some(ApiFnDoc {
            api: if api_macro_info.api_token.is_none() {
                item_fn.sig.ident.to_string()
            } else {
                api_macro_info.api_token.unwrap().value_token.value()
            },
            group: if let Some(group_token) = api_macro_info.group_token {
                group_token.value_token.value()
            } else {
                "Default".to_string()
            },
        }),
        use_crate: None,
    };
    Ok(api_fn)
}

pub fn parse_fn_item_with_path(item_fn: &ItemFn, path_buf: PathBuf) -> Result<Option<(String, ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>>)>, Box<dyn
Error>> {
    let fn_full_crate_path;
    //获取函数上的标记宏
    for attr in &item_fn.attrs {
        let meta = &attr.meta;
        if let Ok(meta_list) = meta.require_list() {
            let path = &meta_list.path;
            if path.is_ident("post") {
                let api_fn = parse_api_info(item_fn, attr, "post")?;
                fn_full_crate_path = gen_fn_full_crate_path(&path_buf, api_fn.api_fn_name.clone());
                return Ok(Some((fn_full_crate_path, api_fn)));
            } else if path.is_ident("get") {
                let api_fn = parse_api_info(item_fn, attr, "get")?;
                fn_full_crate_path = gen_fn_full_crate_path(&path_buf, api_fn.api_fn_name.clone());
                return Ok(Some((fn_full_crate_path, api_fn)));
            } else if path.is_ident("put") {
                let api_fn = parse_api_info(item_fn, attr, "put")?;
                fn_full_crate_path = gen_fn_full_crate_path(&path_buf, api_fn.api_fn_name.clone());
                return Ok(Some((fn_full_crate_path, api_fn)));
            } else if path.is_ident("delete") {
                let api_fn = parse_api_info(item_fn, attr, "delete")?;
                fn_full_crate_path = gen_fn_full_crate_path(&path_buf, api_fn.api_fn_name.clone());
                return Ok(Some((fn_full_crate_path, api_fn)));
            } else {
                //不是api宏
            }
        }
    }
    Ok(None)
}

fn gen_fn_full_crate_path(path_buf: &PathBuf, fn_name: String) -> String {
    if let Some(src_index) = path_buf.iter().position(|component| component == "src") {
        // 截取src目录之后的路径部分
        let after_src_path = path_buf.iter().skip(src_index + 1).collect::<PathBuf>();

        // 对于最后的文件, 我们需要单独处理, 去除扩展名
        let mut module_parts: Vec<String> = after_src_path
            .iter()
            .take_while(|&component| component != after_src_path.file_name().unwrap())
            .map(|s| {
                s.to_string_lossy()
                    .into_owned()
                    .replace("\\", "::")
                    .replace("/", "::")
            })
            .collect();

        // 处理文件名去掉.rs扩展
        if let Some(file_stem) = after_src_path.file_stem() {
            module_parts.push(file_stem.to_string_lossy().into_owned());
        }

        // 将各部分拼接成完整的模块路径
        let module_path = module_parts.join("::");

        // 添加前缀
        return format!("crate::{}::{}", module_path, fn_name);
    }
    "".to_string()
}

pub fn parse_fn_item_in_mod(
    fns: &mut HashMap<String, ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>>>,
    item_mod: &ItemMod,
    mod_name: &str,
    path_buf: PathBuf,
) {
    //先获取全部的use,防止有些文件没有进行rustfmt
    let mut item_uses: Vec<ItemUse> = vec![];
    for content in item_mod.content.iter() {
        for item in content.clone().1.iter() {
            match item {
                Item::Use(item_use) => {
                    item_uses.push(item_use.clone());
                }
                _ => {}
            }
        }
    }
    for content in item_mod.content.iter() {
        for fn_item in content.clone().1.iter() {
            match fn_item {
                Item::Fn(fn_item) => {
                    if let Some(parsed) =
                        parse_fn_item_with_mod(fn_item, path_buf.clone(), mod_name).unwrap()
                    {
                        let (fn_name, mut api_fn) = parsed;
                        eprintln!("add fn :{:?}", fn_name);
                        api_fn.use_crate = Some(item_uses.clone());
                        fns.insert(fn_name, api_fn);
                    }
                }
                Item::Mod(mod_item) => {
                    parse_fn_item_in_mod(
                        fns,
                        mod_item,
                        format!("{}::{}", mod_name, mod_item.ident.to_string()).as_str(),
                        path_buf.clone(),
                    );
                }
                _ => {}
            }
        }
    }
}


fn parse_fn_item_with_mod(
    item_fn: &ItemFn,
    path_buf: PathBuf,
    mod_name: &str,
) -> Result<Option<(String, ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>>)>, Box<dyn Error>> {
    let fn_full_crate_path;
    //获取函数上的标记宏
    for attr in &item_fn.attrs {
        let meta = &attr.meta;
        if let Ok(meta_list) = meta.require_list() {
            let path = &meta_list.path;
            if path.is_ident("post") {
                let api_fn = parse_api_info(item_fn, attr, "post")?;
                fn_full_crate_path =
                    gen_fn_full_crate_path_in_mod(&path_buf, api_fn.api_fn_name.clone(), mod_name);
                return Ok(Some((fn_full_crate_path, api_fn)));
            } else if path.is_ident("get") {
                let api_fn = parse_api_info(item_fn, attr, "get")?;
                fn_full_crate_path =
                    gen_fn_full_crate_path_in_mod(&path_buf, api_fn.api_fn_name.clone(), mod_name);
                return Ok(Some((fn_full_crate_path, api_fn)));
            } else if path.is_ident("put") {
                let api_fn = parse_api_info(item_fn, attr, "put")?;
                fn_full_crate_path =
                    gen_fn_full_crate_path_in_mod(&path_buf, api_fn.api_fn_name.clone(), mod_name);
                return Ok(Some((fn_full_crate_path, api_fn)));
            } else if path.is_ident("delete") {
                let api_fn = parse_api_info(item_fn, attr, "delete")?;
                fn_full_crate_path =
                    gen_fn_full_crate_path_in_mod(&path_buf, api_fn.api_fn_name.clone(), mod_name);
                return Ok(Some((fn_full_crate_path, api_fn)));
            } else {
                //不是api宏
            }
        }
    }
    Ok(None)
}


fn gen_fn_full_crate_path_in_mod(path_buf: &PathBuf, fn_name: String, mod_name: &str) -> String {
    if let Some(src_index) = path_buf.iter().position(|component| component == "src") {
        // 截取src目录之后的路径部分
        let after_src_path = path_buf.iter().skip(src_index + 1).collect::<PathBuf>();

        // 对于最后的文件 去除扩展名
        let module_parts: Vec<String> = after_src_path
            .iter()
            .take_while(|&component| component != after_src_path.file_name().unwrap())
            .map(|s| {
                s.to_string_lossy()
                    .into_owned()
                    .replace("\\", "::")
                    .replace("/", "::")
            })
            .collect();

        // 将各部分拼接成完整的模块路径
        let module_path = module_parts.join("::");

        // 添加前缀
        return format!("crate::{}::{}::{}", module_path, mod_name, fn_name);
    }
    "".to_string()
}
