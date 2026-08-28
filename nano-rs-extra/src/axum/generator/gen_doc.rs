use crate::axum::generator::cache::{DocSchemaCache, FileFingerprint};
#[cfg(feature = "utoipa_axum")]
use crate::axum::generator::parse_utoipa_info;
use crate::axum::generator::{AxumGen, write_if_changed};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    Attribute, FnArg, GenericArgument, Item, ItemUse, LitStr, Pat, Path as SynPath, PathArguments,
    Type, TypePath, UseGroup, UseName, UsePath, UseRename, UseTree, Visibility, parse_str,
};

use utoipa::openapi::{ExternalDocs, Info, Object, SecurityRequirement, Server, Tag};

use nano_rs_build::api_fn::ApiFn;
use nano_rs_build::api_gen::GenDoc;
use nano_rs_build::api_parse::{CrateContext, resolve_crate_context};

pub struct AxumGenDoc {
    pub info: Info,
    pub servers: Vec<Server>,
    pub security: Vec<SecurityRequirement>,
    pub tags: Vec<Tag>,
    pub external_docs: Option<ExternalDocs>,
    pub extensions: Object,
}

#[derive(Clone)]
enum ParamSource {
    Path,
    Query,
}

#[derive(Clone)]
enum ParamSpec {
    Structured {
        ty: Type,
    },
    Primitive {
        name: String,
        ty: Type,
        source: ParamSource,
    },
}

#[derive(Clone)]
enum RequestBodySpec {
    Json(Type),
    Form(Type),
}

#[derive(Clone)]
enum ResponseSpec {
    Body(Type),
    StatusOnly,
}

#[derive(Clone)]
struct AutoPathSpec {
    method: String,
    path: String,
    tag: String,
    summary: String,
    description: String,
    params: Vec<ParamSpec>,
    request_body: Option<RequestBodySpec>,
    response: ResponseSpec,
}

impl GenDoc for AxumGenDoc {
    fn gen_doc(
        &self,
        rs_files: Vec<PathBuf>,
        path_buf: PathBuf,
        api_fns: HashMap<
            String,
            ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
        >,
        cache_enabled: bool,
    ) {
        eprintln!("AxumGenRoute gen_doc in {:?}", path_buf);
        let mut schema_cache: DocSchemaCache = if cache_enabled {
            DocSchemaCache::load(path_buf.as_path())
        } else {
            DocSchemaCache::default()
        };
        let mut struct_paths: BTreeSet<String> = BTreeSet::new();
        let mut enum_paths: BTreeSet<String> = BTreeSet::new();
        let mut crate_cache: HashMap<PathBuf, CrateContext> = HashMap::new();
        self.parse_to_schema(
            &mut struct_paths,
            &mut enum_paths,
            rs_files,
            &mut schema_cache,
            path_buf.as_path(),
            &mut crate_cache,
            cache_enabled,
        );
        if cache_enabled {
            if let Err(err) = schema_cache.save(path_buf.as_path()) {
                eprintln!("failed to persist doc cache: {err}");
            }
        }
        let docs = path_buf.join(self.get_doc_file_path());
        let mut api_fns = api_fns;
        #[cfg(feature = "utoipa_axum")]
        for api_fn in api_fns.values_mut() {
            if let Err(err) = parse_utoipa_info(api_fn) {
                panic!("{err}");
            }
        }

        let mut api_fns_keys: Vec<String> = api_fns.keys().cloned().collect();
        api_fns_keys.sort();
        let mut fns_code: Vec<TokenStream> = vec![];
        let mut auto_doc_fn_code: Vec<TokenStream> = vec![];
        for (index, fn_name) in api_fns_keys.iter().enumerate() {
            if let Some(api_fn) = api_fns.get(fn_name) {
                if api_fn.has_utoipa_path {
                    let type_path: TypePath =
                        parse_str(fn_name.as_str()).expect("Failed to parse type path");
                    fns_code.push(quote! {
                        #type_path
                    });
                    continue;
                }

                if let Some(auto_spec) = self.infer_auto_path_spec(fn_name.as_str(), api_fn) {
                    let ident_name = format!(
                        "__nano_auto_doc_{}_{}",
                        index,
                        self.sanitize_ident(api_fn.api_fn_name.as_str())
                    );
                    let ident =
                        syn::Ident::new(ident_name.as_str(), proc_macro2::Span::call_site());
                    auto_doc_fn_code.push(self.gen_auto_path_fn_tokens(&ident, auto_spec));
                    fns_code.push(quote! {
                        #ident
                    });
                }
            }
        }

        let mut tags_code = vec![];
        for tag in &self.tags {
            let name = LitStr::new(tag.name.as_str(), Span::call_site());
            let description_value = tag.description.as_deref().unwrap_or("");
            let description = LitStr::new(description_value, Span::call_site());
            tags_code.push(quote! {
                (name = #name, description = #description),
            })
        }

        let mut components_code = vec![];
        for key in &struct_paths {
            let type_path: TypePath = parse_str(key.as_str())
                .unwrap_or_else(|_| panic!("Failed to parse type path -> {key}"));
            components_code.push(quote! {
                #type_path
            });
        }
        for key in &enum_paths {
            let type_path: TypePath = parse_str(key.as_str())
                .unwrap_or_else(|_| panic!("Failed to parse type path -> {key}"));
            components_code.push(quote! {
                #type_path
            });
        }

        let title = LitStr::new(self.info.title.as_str(), Span::call_site());
        let description_value = self.info.description.as_deref().unwrap_or("");
        let description = LitStr::new(description_value, Span::call_site());
        let version = LitStr::new(self.info.version.as_str(), Span::call_site());
        let license = self.info.license.as_ref();
        let license_name_value = license.map(|item| item.name.as_str()).unwrap_or("");
        let license_name = LitStr::new(license_name_value, Span::call_site());
        let license_url_value = license.and_then(|item| item.url.as_deref()).unwrap_or("");
        let license_url = LitStr::new(license_url_value, Span::call_site());
        let contact = self.info.contact.as_ref();
        let contact_name_value = contact.and_then(|item| item.name.as_deref()).unwrap_or("");
        let contact_name = LitStr::new(contact_name_value, Span::call_site());
        let contact_email_value = contact.and_then(|item| item.email.as_deref()).unwrap_or("");
        let contact_email = LitStr::new(contact_email_value, Span::call_site());
        let contact_url_value = contact.and_then(|item| item.url.as_deref()).unwrap_or("");
        let contact_url = LitStr::new(contact_url_value, Span::call_site());
        let info_code = quote! {
            info(
                title = #title,
                description = #description,
                version = #version,
                license(
                    name = #license_name,
                    url = #license_url,
                ),
                contact(
                    name = #contact_name,
                    email = #contact_email,
                    url = #contact_url
                ),
            )
        };

        let mut servers_code = vec![];
        for server in self.servers.iter() {
            let server_url = LitStr::new(server.url.as_str(), Span::call_site());
            let server_description_value = server.description.as_deref().unwrap_or("");
            let server_description = LitStr::new(server_description_value, Span::call_site());
            servers_code.push(quote! {
               (url = #server_url, description = #server_description),
            });
        }

        let doc_code = quote! {
            /// Code generated by nano-rs. DO NOT EDIT.
            use utoipa::OpenApi;

            #(#auto_doc_fn_code)*

            #[derive(OpenApi)]
            #[openapi(
                #info_code,
                paths(#(#fns_code),*),
                components(schemas(#(#components_code),*)),
                servers(
                    #(#servers_code)*
                ),
                tags(
                    #(#tags_code)*
                )
            )]
            pub struct GenApi{}
        };
        let syntax_tree: syn::File = syn::parse2(doc_code).unwrap();
        let formatted = prettyplease::unparse(&syntax_tree);
        let _ =
            write_if_changed(docs.as_path(), formatted.as_str()).expect("write doc file failed");
        // let output = Command::new("rustfmt")
        //     .arg(docs.as_path())
        //     .output()
        //     .expect("Failed to execute rustfmt");
        // if !output.status.success() {
        //     eprintln!(
        //         "Rustfmt failed: {}",
        //         String::from_utf8_lossy(&output.stderr)
        //     );
        // }
    }
}

impl AxumGenDoc {
    pub fn new() -> AxumGenDocBuilder {
        AxumGenDocBuilder::default()
    }

    fn infer_auto_path_spec(
        &self,
        fn_name: &str,
        api_fn: &ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
    ) -> Option<AutoPathSpec> {
        if api_fn.path.trim().is_empty() {
            return None;
        }
        let mut params = Vec::new();
        let mut request_body: Option<RequestBodySpec> = None;
        if let Some(inputs) = api_fn.inputs.as_ref() {
            for arg in inputs.iter() {
                let FnArg::Typed(pat_type) = arg else {
                    continue;
                };
                let Type::Path(type_path) = &*pat_type.ty else {
                    continue;
                };
                let Some(extractor_segment) = type_path.path.segments.last() else {
                    continue;
                };
                let extractor = extractor_segment.ident.to_string();
                let Some(inner_ty) = self
                    .first_generic_type(extractor_segment)
                    .and_then(|inner| self.resolve_type_for_doc(&inner, fn_name, api_fn))
                else {
                    continue;
                };
                match extractor.as_str() {
                    "Path" => {
                        if self.is_primitive_type(&inner_ty) {
                            let name = self
                                .extract_pat_binding_name(&pat_type.pat)
                                .unwrap_or_else(|| "param".to_string());
                            params.push(ParamSpec::Primitive {
                                name,
                                ty: inner_ty,
                                source: ParamSource::Path,
                            });
                        } else {
                            params.push(ParamSpec::Structured { ty: inner_ty });
                        }
                    }
                    "Query" => {
                        if self.is_primitive_type(&inner_ty) {
                            let name = self
                                .extract_pat_binding_name(&pat_type.pat)
                                .unwrap_or_else(|| "query".to_string());
                            params.push(ParamSpec::Primitive {
                                name,
                                ty: inner_ty,
                                source: ParamSource::Query,
                            });
                        } else {
                            params.push(ParamSpec::Structured { ty: inner_ty });
                        }
                    }
                    "Json" => {
                        if request_body.is_none() {
                            request_body = Some(RequestBodySpec::Json(inner_ty));
                        }
                    }
                    "Form" => {
                        if request_body.is_none() {
                            request_body = Some(RequestBodySpec::Form(inner_ty));
                        }
                    }
                    // ignore State and all unknown extractors silently
                    _ => {}
                }
            }
        }
        let response = self.infer_response_spec(fn_name, api_fn);
        let tag = api_fn
            .api_fn_doc
            .as_ref()
            .map(|doc| doc.api_group.clone())
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "Default".to_string());
        let summary = api_fn
            .api_fn_doc
            .as_ref()
            .map(|doc| doc.api.clone())
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| api_fn.api_fn_name.clone());
        let description = api_fn
            .api_fn_doc
            .as_ref()
            .map(|doc| doc.api_desc.clone())
            .unwrap_or_default();
        Some(AutoPathSpec {
            method: api_fn.method.clone(),
            path: api_fn.path.clone(),
            tag,
            summary,
            description,
            params,
            request_body,
            response,
        })
    }

    fn infer_response_spec(
        &self,
        fn_name: &str,
        api_fn: &ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
    ) -> ResponseSpec {
        let Some(output) = api_fn.output.as_ref() else {
            return ResponseSpec::StatusOnly;
        };
        let Ok(output_ty) = parse_str::<Type>(output.as_str()) else {
            return ResponseSpec::StatusOnly;
        };
        let Some(body_ty) = self.extract_rest_resp_body_type(&output_ty) else {
            return ResponseSpec::StatusOnly;
        };
        let Some(body_ty) = self.resolve_type_for_doc(&body_ty, fn_name, api_fn) else {
            return ResponseSpec::StatusOnly;
        };
        ResponseSpec::Body(body_ty)
    }

    fn extract_rest_resp_body_type(&self, ty: &Type) -> Option<Type> {
        let Type::Path(type_path) = ty else {
            return None;
        };
        let last = type_path.path.segments.last()?;
        let ident = last.ident.to_string();
        if ident == "Result" {
            let inner = self.first_generic_type(last)?;
            return self.extract_rest_resp_body_type(&inner);
        }
        if ident == "RestResp" {
            return self.first_generic_type(last);
        }
        None
    }

    fn gen_auto_path_fn_tokens(&self, ident: &syn::Ident, spec: AutoPathSpec) -> TokenStream {
        let method_ident = parse_str::<syn::Ident>(spec.method.as_str())
            .unwrap_or_else(|_| syn::Ident::new("get", Span::call_site()));
        let path = LitStr::new(spec.path.as_str(), Span::call_site());
        let tag = LitStr::new(spec.tag.as_str(), Span::call_site());
        let summary = LitStr::new(spec.summary.as_str(), Span::call_site());
        let description_code = if spec.description.trim().is_empty() {
            quote! {}
        } else {
            let description = LitStr::new(spec.description.as_str(), Span::call_site());
            quote! {
                description = #description,
            }
        };
        let mut params_tokens: Vec<TokenStream> = Vec::new();
        for param in spec.params {
            match param {
                ParamSpec::Structured { ty } => {
                    params_tokens.push(quote! { #ty });
                }
                ParamSpec::Primitive { name, ty, source } => {
                    let name = LitStr::new(name.as_str(), Span::call_site());
                    let source = match source {
                        ParamSource::Path => quote!(Path),
                        ParamSource::Query => quote!(Query),
                    };
                    params_tokens.push(quote! {
                        (#name = #ty, #source)
                    });
                }
            }
        }
        let params_code = if params_tokens.is_empty() {
            quote! {}
        } else {
            quote! {
                params(#(#params_tokens),*),
            }
        };
        let request_body_code = match spec.request_body {
            Some(RequestBodySpec::Json(ty)) => {
                quote! {
                    request_body = #ty,
                }
            }
            Some(RequestBodySpec::Form(ty)) => {
                quote! {
                    request_body(content = #ty, content_type = "application/x-www-form-urlencoded"),
                }
            }
            None => quote! {},
        };
        let response_code = match spec.response {
            ResponseSpec::Body(ty) => {
                quote! {
                    responses(
                        (status = 200, body = #ty),
                        (status = 500, description = "Internal Server Error")
                    )
                }
            }
            ResponseSpec::StatusOnly => {
                quote! {
                    responses(
                        (status = 200)
                    )
                }
            }
        };
        quote! {
            #[utoipa::path(
                #method_ident,
                path = #path,
                tag = #tag,
                summary = #summary,
                #description_code
                #params_code
                #request_body_code
                #response_code
            )]
            fn #ident() {}
        }
    }

    fn resolve_type_for_doc(
        &self,
        ty: &Type,
        fn_name: &str,
        api_fn: &ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
    ) -> Option<Type> {
        let mut ty = ty.clone();
        self.rewrite_type_paths(&mut ty, fn_name, api_fn);
        Some(ty)
    }

    fn rewrite_type_paths(
        &self,
        ty: &mut Type,
        fn_name: &str,
        api_fn: &ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
    ) {
        match ty {
            Type::Path(type_path) => {
                for segment in type_path.path.segments.iter_mut() {
                    if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
                        for arg in args.args.iter_mut() {
                            if let GenericArgument::Type(arg_ty) = arg {
                                self.rewrite_type_paths(arg_ty, fn_name, api_fn);
                            }
                        }
                    }
                }
                if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
                    let ident = type_path.path.segments[0].ident.to_string();
                    if self.should_resolve_ident(ident.as_str()) {
                        if let Some(full) = self.resolve_type_ident(ident.as_str(), fn_name, api_fn)
                        {
                            if let Ok(mut path) = parse_str::<SynPath>(full.as_str()) {
                                if let Some(last) = path.segments.last_mut() {
                                    last.arguments = type_path.path.segments[0].arguments.clone();
                                }
                                type_path.path = path;
                            }
                        }
                    }
                }
            }
            Type::Reference(reference) => {
                self.rewrite_type_paths(reference.elem.as_mut(), fn_name, api_fn);
            }
            Type::Tuple(tuple) => {
                for item in tuple.elems.iter_mut() {
                    self.rewrite_type_paths(item, fn_name, api_fn);
                }
            }
            _ => {}
        }
    }

    fn resolve_type_ident(
        &self,
        ident: &str,
        fn_name: &str,
        api_fn: &ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
    ) -> Option<String> {
        if let Some(use_items) = api_fn.use_crate.as_ref() {
            if let Some(full) = self.get_full_crate_name(ident.to_string(), use_items) {
                return Some(full);
            }
        }
        if let Some((module, _)) = fn_name.rsplit_once("::") {
            return Some(format!("{module}::{ident}"));
        }
        if !api_fn.crate_prefix.is_empty() {
            return Some(format!("{}::{}", api_fn.crate_prefix, ident));
        }
        None
    }

    fn should_resolve_ident(&self, ident: &str) -> bool {
        !self.is_primitive_type_name(ident) && !self.is_container_type_name(ident)
    }

    fn is_primitive_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Path(type_path)
                if type_path.qself.is_none() && type_path.path.segments.len() == 1 =>
            {
                let ident = type_path.path.segments[0].ident.to_string();
                self.is_primitive_type_name(ident.as_str())
            }
            _ => false,
        }
    }

    fn is_primitive_type_name(&self, ident: &str) -> bool {
        matches!(
            ident,
            "String"
                | "str"
                | "bool"
                | "char"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "f32"
                | "f64"
        )
    }

    fn is_container_type_name(&self, ident: &str) -> bool {
        matches!(
            ident,
            "Vec"
                | "Option"
                | "Result"
                | "HashMap"
                | "HashSet"
                | "BTreeMap"
                | "BTreeSet"
                | "Box"
                | "Arc"
                | "Rc"
                | "Cow"
        )
    }

    fn first_generic_type(&self, segment: &syn::PathSegment) -> Option<Type> {
        if let PathArguments::AngleBracketed(args) = &segment.arguments {
            for arg in args.args.iter() {
                if let GenericArgument::Type(ty) = arg {
                    return Some(ty.clone());
                }
            }
        }
        None
    }

    fn extract_pat_binding_name(&self, pat: &Pat) -> Option<String> {
        match pat {
            Pat::Ident(ident) => {
                let name = ident.ident.to_string();
                let cleaned = name.trim_start_matches('_').to_string();
                if cleaned.is_empty() {
                    None
                } else {
                    Some(cleaned)
                }
            }
            Pat::TupleStruct(tuple_struct) => tuple_struct
                .elems
                .iter()
                .find_map(|elem| self.extract_pat_binding_name(elem)),
            Pat::Struct(st) => st
                .fields
                .iter()
                .find_map(|field| self.extract_pat_binding_name(&field.pat)),
            Pat::Reference(reference) => self.extract_pat_binding_name(reference.pat.as_ref()),
            Pat::Type(pat_type) => self.extract_pat_binding_name(pat_type.pat.as_ref()),
            _ => None,
        }
    }

    fn sanitize_ident(&self, value: &str) -> String {
        let mut result = String::new();
        for ch in value.chars() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                result.push(ch);
            } else {
                result.push('_');
            }
        }
        if result.is_empty()
            || result
                .chars()
                .next()
                .map(|ch| ch.is_ascii_digit())
                .unwrap_or(false)
        {
            result.insert(0, '_');
        }
        result
    }

    fn parse_to_schema(
        &self,
        struct_paths: &mut BTreeSet<String>,
        enum_paths: &mut BTreeSet<String>,
        rs_files: Vec<PathBuf>,
        cache: &mut DocSchemaCache,
        base_path: &Path,
        crate_cache: &mut HashMap<PathBuf, CrateContext>,
        cache_enabled: bool,
    ) {
        let mut crate_reexports: HashMap<PathBuf, HashSet<String>> = HashMap::new();
        for rs_file in rs_files {
            let fingerprint = if cache_enabled {
                FileFingerprint::from_path(rs_file.as_path()).ok()
            } else {
                None
            };
            if cache_enabled {
                if let Some(fp) = fingerprint.as_ref() {
                    if let Some(entry) = cache.get(base_path, rs_file.as_path(), fp) {
                        for value in &entry.structs {
                            struct_paths.insert(value.clone());
                        }
                        for value in &entry.enums {
                            enum_paths.insert(value.clone());
                        }
                        continue;
                    }
                }
            }

            let crate_ctx = resolve_crate_context(rs_file.as_path(), base_path, crate_cache);
            let root_reexports = self.load_root_reexports(&crate_ctx, &mut crate_reexports);
            let (structs, enums) =
                self.extract_schemas(&crate_ctx, rs_file.as_path(), &root_reexports);
            for value in &structs {
                struct_paths.insert(value.clone());
            }
            for value in &enums {
                enum_paths.insert(value.clone());
            }
            if cache_enabled {
                if let Some(fp) = fingerprint {
                    cache.update(base_path, rs_file.as_path(), &fp, structs, enums);
                }
            }
        }
    }

    fn extract_schemas(
        &self,
        crate_ctx: &CrateContext,
        rs_file: &Path,
        root_reexports: &HashSet<String>,
    ) -> (Vec<String>, Vec<String>) {
        let src = fs::read_to_string(rs_file).expect("read file error");
        let syntax_tree = syn::parse_file(&src).expect("parse file error");
        let crate_path = self.parse_path_to_crate(crate_ctx, rs_file);
        let mut module_stack: Vec<String> = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        self.collect_items(
            &syntax_tree.items,
            &crate_path,
            crate_ctx.code_prefix.as_str(),
            root_reexports,
            &mut module_stack,
            &mut structs,
            &mut enums,
        );
        (structs, enums)
    }

    fn collect_items(
        &self,
        items: &[Item],
        crate_path: &str,
        crate_prefix: &str,
        root_reexports: &HashSet<String>,
        module_stack: &mut Vec<String>,
        structs: &mut Vec<String>,
        enums: &mut Vec<String>,
    ) {
        for item in items {
            match item {
                Item::Struct(item_struct) => {
                    if Self::has_to_schema(&item_struct.attrs)
                        && item_struct.generics.params.is_empty()
                    {
                        let ident = item_struct.ident.to_string();
                        let path = if root_reexports.contains(&ident) {
                            format!("{crate_prefix}::{ident}")
                        } else {
                            Self::build_schema_path(crate_path, module_stack, ident)
                        };
                        structs.push(path);
                    }
                }
                Item::Enum(item_enum) => {
                    if Self::has_to_schema(&item_enum.attrs)
                        && item_enum.generics.params.is_empty()
                    {
                        let ident = item_enum.ident.to_string();
                        let path = if root_reexports.contains(&ident) {
                            format!("{crate_prefix}::{ident}")
                        } else {
                            Self::build_schema_path(crate_path, module_stack, ident)
                        };
                        enums.push(path);
                    }
                }
                Item::Mod(item_mod) => {
                    if let Some((_, nested_items)) = &item_mod.content {
                        module_stack.push(item_mod.ident.to_string());
                        self.collect_items(
                            nested_items,
                            crate_path,
                            crate_prefix,
                            root_reexports,
                            module_stack,
                            structs,
                            enums,
                        );
                        module_stack.pop();
                    }
                }
                _ => {}
            }
        }
    }

    fn load_root_reexports(
        &self,
        crate_ctx: &CrateContext,
        cache: &mut HashMap<PathBuf, HashSet<String>>,
    ) -> HashSet<String> {
        if let Some(existing) = cache.get(&crate_ctx.root) {
            return existing.clone();
        }
        let mut exports = HashSet::new();
        let lib_rs = crate_ctx.root.join("src").join("lib.rs");
        if let Ok(src) = fs::read_to_string(lib_rs) {
            if let Ok(syntax_tree) = syn::parse_file(&src) {
                for item in syntax_tree.items {
                    if let Item::Use(item_use) = item {
                        if matches!(item_use.vis, Visibility::Public(_)) {
                            Self::collect_use_export_names(&item_use.tree, &mut exports);
                        }
                    }
                }
            }
        }
        cache.insert(crate_ctx.root.clone(), exports.clone());
        exports
    }

    fn collect_use_export_names(tree: &UseTree, names: &mut HashSet<String>) {
        match tree {
            UseTree::Name(UseName { ident }) => {
                names.insert(ident.to_string());
            }
            UseTree::Rename(UseRename { rename, .. }) => {
                names.insert(rename.to_string());
            }
            UseTree::Group(UseGroup { items, .. }) => {
                for item in items {
                    Self::collect_use_export_names(item, names);
                }
            }
            UseTree::Path(UsePath { tree, .. }) => {
                Self::collect_use_export_names(tree, names);
            }
            _ => {}
        }
    }

    fn build_schema_path(crate_path: &str, module_stack: &[String], ident: String) -> String {
        let mut path = crate_path.to_owned();
        for segment in module_stack {
            path.push_str("::");
            path.push_str(segment);
        }
        path.push_str("::");
        path.push_str(&ident);
        path
    }

    fn has_to_schema(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            if !attr.path().is_ident("derive") {
                return false;
            }
            attr.parse_args_with(Punctuated::<SynPath, Comma>::parse_terminated)
                .map(|paths| {
                    paths.iter().any(|path| {
                        path.segments
                            .last()
                            .map(|segment| segment.ident == "ToSchema")
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        })
    }

    fn parse_path_to_crate(&self, crate_ctx: &CrateContext, rs_file: &Path) -> String {
        let relative = rs_file.strip_prefix(&crate_ctx.root).unwrap_or(rs_file);
        let mut segments: Vec<String> = Vec::new();
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
                            segments.push(stem.to_string());
                        }
                    } else {
                        segments.push(part.to_string());
                    }
                }
            }
        }
        if segments.is_empty() {
            crate_ctx.code_prefix.clone()
        } else {
            format!("{}::{}", crate_ctx.code_prefix, segments.join("::"))
        }
    }
}

impl AxumGen for AxumGenDoc {}

pub struct AxumGenDocBuilder {
    pub info: Info,
    pub servers: Vec<Server>,
    pub security: Vec<SecurityRequirement>,
    pub tags: Vec<Tag>,
    pub external_docs: Option<ExternalDocs>,
    pub extensions: Object,
}

impl AxumGenDocBuilder {
    pub fn build(self) -> AxumGenDoc {
        AxumGenDoc {
            info: self.info,
            servers: self.servers,
            security: self.security,
            tags: self.tags,
            external_docs: self.external_docs,
            extensions: self.extensions,
        }
    }

    pub fn set_info(mut self, info: Info) -> Self {
        self.info = info;
        self
    }

    pub fn add_extensions(mut self, extensions: Object) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn add_external_doc(mut self, external_docs: ExternalDocs) -> Self {
        self.external_docs = Some(external_docs);
        self
    }

    pub fn add_server(mut self, server: Server) -> Self {
        self.servers.push(server);
        self
    }

    pub fn add_tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn add_security(mut self, security_requirement: SecurityRequirement) -> Self {
        self.security.push(security_requirement);
        self
    }
}

impl Default for AxumGenDocBuilder {
    fn default() -> Self {
        AxumGenDocBuilder {
            info: Default::default(),
            servers: vec![],
            security: vec![],
            tags: vec![],
            external_docs: None,
            extensions: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use syn::{GenericArgument, parse_file};

    #[test]
    fn rewrite_type_paths_preserves_generic_arguments() {
        let doc = AxumGenDoc::new().build();
        let mut api_fn: ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>> =
            ApiFn::default();
        api_fn.use_crate = Some(vec![syn::parse_quote!(use basic_model::{PageData, Pet};)]);

        let mut ty = parse_str::<Type>("PageData<Pet>").expect("parse type");
        doc.rewrite_type_paths(&mut ty, "crate::api::handler", &api_fn);

        let Type::Path(type_path) = ty else {
            panic!("expected path type");
        };
        assert_eq!(
            type_path
                .path
                .segments
                .first()
                .expect("first segment")
                .ident
                .to_string(),
            "basic_model"
        );
        let last = type_path.path.segments.last().expect("last segment");
        assert_eq!(last.ident.to_string(), "PageData");

        let PathArguments::AngleBracketed(args) = &last.arguments else {
            panic!("expected generic args");
        };
        let Some(GenericArgument::Type(Type::Path(inner_ty))) = args.args.first() else {
            panic!("expected inner type path");
        };
        assert_eq!(
            inner_ty
                .path
                .segments
                .last()
                .expect("inner last segment")
                .ident
                .to_string(),
            "Pet"
        );
    }

    #[test]
    fn collect_items_skips_generic_schemas() {
        let doc = AxumGenDoc::new().build();
        let syntax_tree = parse_file(
            r#"
            use utoipa::ToSchema;

            #[derive(ToSchema)]
            struct PageData<T> {
                data: Vec<T>,
            }

            #[derive(ToSchema)]
            struct Pet {
                id: i64,
            }

            #[derive(ToSchema)]
            enum ResultData<T> {
                Value(T),
            }

            #[derive(ToSchema)]
            enum PetType {
                Dog,
                Cat,
            }
        "#,
        )
        .expect("parse file");

        let mut module_stack = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let reexports: HashSet<String> = HashSet::new();
        doc.collect_items(
            &syntax_tree.items,
            "crate::model",
            "crate",
            &reexports,
            &mut module_stack,
            &mut structs,
            &mut enums,
        );

        assert_eq!(structs, vec!["crate::model::Pet".to_string()]);
        assert_eq!(enums, vec!["crate::model::PetType".to_string()]);
    }
}
