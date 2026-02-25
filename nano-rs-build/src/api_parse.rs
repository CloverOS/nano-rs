use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, Expr, FnArg, Item, ItemFn, ItemMod, ItemUse, Lit, Meta};

use crate::api_doc::ApiFnDoc;
use crate::api_fn::ApiFn;
use crate::api_syn::ApiMacroInfo;

#[derive(Clone, Debug)]
pub struct CrateContext {
    pub root: PathBuf,
    pub package_name: String,
    pub code_prefix: String,
    pub is_base_crate: bool,
}

pub fn resolve_crate_context(
    file: &Path,
    base_path: &Path,
    cache: &mut HashMap<PathBuf, CrateContext>,
) -> CrateContext {
    let crate_root = find_crate_root(file).unwrap_or_else(|| base_path.to_path_buf());
    if let Some(existing) = cache.get(&crate_root) {
        return existing.clone();
    }
    let package_name = read_crate_name(&crate_root)
        .or_else(|| {
            crate_root
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "crate".to_string());
    let is_base_crate = crate_root == base_path;
    let code_prefix = if is_base_crate {
        "crate".to_string()
    } else {
        package_name.clone()
    };
    let ctx = CrateContext {
        root: crate_root.clone(),
        package_name,
        code_prefix,
        is_base_crate,
    };
    cache.insert(crate_root, ctx.clone());
    ctx
}

fn find_crate_root(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent();
    while let Some(dir) = current {
        let cargo = dir.join("Cargo.toml");
        if cargo.exists() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

fn read_crate_name(crate_root: &Path) -> Option<String> {
    let cargo_toml_path = crate_root.join("Cargo.toml");
    let content = fs::read_to_string(cargo_toml_path).ok()?;
    let mut in_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package && trimmed.starts_with("name") {
            if let Some((_, value)) = trimmed.split_once('=') {
                let value = value.trim().trim_matches('"');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

pub fn gen_fn_full_crate_path(
    crate_ctx: &CrateContext,
    path_buf: &Path,
    fn_name: String,
    mod_name: Option<&str>,
) -> String {
    let relative = path_buf.strip_prefix(&crate_ctx.root).unwrap_or(path_buf);
    let mut module_parts: Vec<String> = Vec::new();
    let mut inside_src = false;

    for component in relative.components() {
        use std::path::Component;
        match component {
            Component::ParentDir | Component::CurDir | Component::RootDir => {}
            Component::Prefix(_) => {}
            Component::Normal(os_str) => {
                let part = os_str.to_string_lossy();
                if !inside_src {
                    if part == "src" {
                        inside_src = true;
                    }
                    continue;
                }
                if part == "mod.rs" || part == "lib.rs" || part == "main.rs" {
                    continue;
                }
                if part.ends_with(".rs") {
                    if mod_name.is_none() {
                        let stem = part.trim_end_matches(".rs");
                        if !stem.is_empty() {
                            module_parts.push(stem.to_string());
                        }
                    }
                } else {
                    module_parts.push(part.to_string());
                }
            }
        }
    }

    if let Some(module) = mod_name {
        module
            .split("::")
            .filter(|segment| !segment.is_empty())
            .for_each(|segment| module_parts.push(segment.to_string()));
    }

    let module_path = module_parts.join("::").trim_matches(':').to_string();
    let prefix = crate_ctx.code_prefix.as_str();

    if module_path.is_empty() {
        format!("{prefix}::{fn_name}")
    } else {
        format!("{prefix}::{}::{}", module_path, fn_name)
    }
}

pub fn gen_file_crate_module_path(crate_ctx: &CrateContext, path_buf: &Path) -> String {
    let relative = path_buf.strip_prefix(&crate_ctx.root).unwrap_or(path_buf);
    let mut module_parts: Vec<String> = Vec::new();
    let mut inside_src = false;

    for component in relative.components() {
        use std::path::Component;
        match component {
            Component::ParentDir | Component::CurDir | Component::RootDir => {}
            Component::Prefix(_) => {}
            Component::Normal(os_str) => {
                let part = os_str.to_string_lossy();
                if !inside_src {
                    if part == "src" {
                        inside_src = true;
                    }
                    continue;
                }
                if part == "mod.rs" || part == "lib.rs" || part == "main.rs" {
                    continue;
                }
                if part.ends_with(".rs") {
                    let stem = part.trim_end_matches(".rs");
                    if !stem.is_empty() {
                        module_parts.push(stem.to_string());
                    }
                } else {
                    module_parts.push(part.to_string());
                }
            }
        }
    }

    if module_parts.is_empty() {
        crate_ctx.code_prefix.clone()
    } else {
        format!("{}::{}", crate_ctx.code_prefix, module_parts.join("::"))
    }
}

pub fn parse_mod_tag_attr(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(name_value) = &attr.meta {
                if let Expr::Lit(lit) = &name_value.value {
                    if let Lit::Str(value) = &lit.lit {
                        let doc_line = value.value();
                        let line = doc_line.trim();
                        if let Some(rest) = line.strip_prefix("@tag") {
                            let mut chars = rest.chars();
                            match chars.next() {
                                Some(ch) if ch.is_whitespace() || ch == ':' => {
                                    let rest = rest.trim_start().trim_start_matches(':').trim_start();
                                    if !rest.is_empty() {
                                        return Some(rest.to_string());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn extract_doc_comments(attrs: &Vec<Attribute>) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let Meta::NameValue(name_value) = &attr.meta {
                    if let Expr::Lit(lit) = &name_value.value {
                        if let Lit::Str(str) = &lit.lit {
                            return Some(str.value().trim().to_string());
                        }
                    }
                }
                None
            } else {
                None
            }
        })
        .collect()
}

pub fn parse_fn_item_in_mod(
    fns: &mut HashMap<
        String,
        ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
    >,
    item_mod: &ItemMod,
    mod_name: &str,
    path_buf: PathBuf,
    crate_ctx: &CrateContext,
    default_tag: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    //先获取全部的use,防止有些文件没有进行rustfmt
    let mut item_uses: Vec<ItemUse> = vec![];
    let Some((_, items)) = &item_mod.content else {
        return Ok(());
    };
    for item in items.iter() {
        if let Item::Use(item_use) = item {
            item_uses.push(item_use.clone());
        }
    }

    for fn_item in items.iter() {
        match fn_item {
            Item::Fn(fn_item) => {
                if let Some(parsed) = parse_fn_item(
                    fn_item,
                    path_buf.clone(),
                    Some(mod_name),
                    crate_ctx,
                    default_tag,
                )? {
                    let (fn_name, mut api_fn) = parsed;
                    eprintln!("add fn in mod :{:?}", fn_name);
                    api_fn.use_crate = Some(item_uses.clone());
                    api_fn.crate_prefix = crate_ctx.code_prefix.clone();
                    api_fn.crate_name = Some(crate_ctx.package_name.clone());
                    fns.insert(fn_name, api_fn);
                }
            }
            Item::Mod(mod_item) => {
                let nested_mod_name = format!("{}::{}", mod_name, mod_item.ident);
                parse_fn_item_in_mod(
                    fns,
                    mod_item,
                    nested_mod_name.as_str(),
                    path_buf.clone(),
                    crate_ctx,
                    default_tag,
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn parse_fn_item(
    item_fn: &ItemFn,
    path_buf: PathBuf,
    mod_name: Option<&str>,
    crate_ctx: &CrateContext,
    default_tag: Option<&str>,
) -> Result<
    Option<(
        String,
        ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
    )>,
    Box<dyn Error>,
> {
    //获取函数上的标记宏
    const METHODS: [&str; 8] = [
        "post", "get", "put", "delete", "patch", "options", "head", "trace",
    ];
    for attr in &item_fn.attrs {
        let meta = &attr.meta;
        if let Ok(meta_list) = meta.require_list() {
            let path = &meta_list.path;
            if let Some(ident) = path.get_ident() {
                let method = ident.to_string();
                if METHODS.contains(&method.as_str()) {
                    let api_fn = parse_api_info(item_fn, attr, method.as_str(), default_tag)?;
                    let fn_full_crate_path = gen_fn_full_crate_path(
                        crate_ctx,
                        &path_buf,
                        api_fn.api_fn_name.clone(),
                        mod_name,
                    );
                    return Ok(Some((fn_full_crate_path, api_fn)));
                }
            }
        }
    }
    Ok(None)
}

pub fn parse_api_info(
    item_fn: &ItemFn,
    attr: &Attribute,
    method: &str,
    default_tag: Option<&str>,
) -> Result<ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>, Box<dyn Error>> {
    let api_macro_info = attr.parse_args::<ApiMacroInfo>()?;
    let has_utoipa_path = has_utoipa_path_attr(&item_fn.attrs);
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
    // extract doc comments
    let docs = extract_doc_comments(&item_fn.attrs);
    let path = if let Some(path) = api_macro_info.path_token {
        path.value_token.value()
    } else {
        "".to_string()
    };
    if path.trim().is_empty() && !has_utoipa_path {
        return Err(format!(
            "missing required 'path' in #[{}(...)] on function '{}'; add path = \"...\" or #[utoipa::path(...)]",
            method, item_fn.sig.ident
        )
        .into());
    }
    let output = match &item_fn.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(ty.to_token_stream().to_string()),
    };
    let api_group = if let Some(tag_token) = api_macro_info.tag_token {
        tag_token.value_token.value()
    } else if let Some(group_token) = api_macro_info.group_token {
        group_token.value_token.value()
    } else if let Some(default_tag) = default_tag {
        default_tag.to_string()
    } else {
        "Default".to_string()
    };

    let api_fn = ApiFn {
        api_fn_name: item_fn.sig.ident.to_string(),
        layers_fn_name: if layers.is_empty() {
            None
        } else {
            Some(layers)
        },
        inputs: Some(item_fn.sig.inputs.clone()),
        path,
        path_group: if let Some(path_group) = api_macro_info.path_group_token {
            path_group.value_token.value()
        } else {
            "".to_string()
        },
        method: method.to_string(),
        public: open_token,
        api_fn_doc: Some(ApiFnDoc {
            api: if api_macro_info.api_token.is_none() {
                if let Some(summary) = docs.first() {
                    summary.to_string()
                } else {
                    item_fn.sig.ident.to_string()
                }
            } else {
                api_macro_info.api_token.unwrap().value_token.value()
            },
            api_desc: if docs.len() > 1 {
                docs.iter()
                    .skip(1)
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                "".to_string()
            },
            api_group,
        }),
        use_crate: None,
        attrs: Some(item_fn.attrs.clone()),
        output,
        has_utoipa_path,
        crate_prefix: String::new(),
        crate_name: None,
    };
    Ok(api_fn)
}

fn has_utoipa_path_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let segments: Vec<_> = attr.meta.path().segments.iter().collect();
        segments.len() > 1 && segments[0].ident == "utoipa" && segments[1].ident == "path"
    })
}

#[cfg(test)]
mod tests {
    use super::parse_mod_tag_attr;
    use syn::Attribute;

    #[test]
    fn parse_mod_tag_attr_accepts_doc_directive() {
        let attrs: Vec<Attribute> = vec![syn::parse_quote!(#[doc = "@tag Store"])];
        assert_eq!(parse_mod_tag_attr(&attrs), Some("Store".to_string()));
    }

    #[test]
    fn parse_mod_tag_attr_accepts_doc_directive_with_colon() {
        let attrs: Vec<Attribute> = vec![syn::parse_quote!(#[doc = "@tag: Store"])];
        assert_eq!(parse_mod_tag_attr(&attrs), Some("Store".to_string()));
    }

    #[test]
    fn parse_mod_tag_attr_ignores_non_directive_like_text() {
        let attrs: Vec<Attribute> = vec![
            syn::parse_quote!(#[doc = "@tagging Store"]),
            syn::parse_quote!(#[doc = "tag: Store"]),
        ];
        assert_eq!(parse_mod_tag_attr(&attrs), None);
    }
}
