use crate::axum::generator::cache::{DocSchemaCache, FileFingerprint};
use proc_macro2::Span;
use quote::quote;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, FnArg, Item, ItemUse, LitStr, Path as SynPath, TypePath, parse_str};
use utoipa::openapi::{ExternalDocs, Info, Object, SecurityRequirement, Server, Tag};

use nano_rs_build::api_fn::ApiFn;
use nano_rs_build::api_gen::GenDoc;
use nano_rs_build::api_parse::{CrateContext, resolve_crate_context};

use crate::axum::generator::AxumGen;

pub struct AxumGenDoc {
    pub info: Info,
    pub servers: Vec<Server>,
    pub security: Vec<SecurityRequirement>,
    pub tags: Vec<Tag>,
    pub external_docs: Option<ExternalDocs>,
    pub extensions: Object,
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
    ) {
        eprintln!("AxumGenRoute gen_doc in {:?}", path_buf);
        let mut schema_cache: DocSchemaCache = DocSchemaCache::load(path_buf.as_path());
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
        );
        if let Err(err) = schema_cache.save(path_buf.as_path()) {
            eprintln!("failed to persist doc cache: {err}");
        }
        let docs = path_buf.join(self.get_doc_file_path());

        let mut api_fns_keys: Vec<_> = api_fns.keys().collect();
        api_fns_keys.sort();
        let mut fns_code = vec![];
        for fn_name in api_fns_keys {
            if let Some(api_fn) = api_fns.get(fn_name) {
                if self.is_utoipa_marco(api_fn) {
                    let type_path: TypePath =
                        parse_str(fn_name.as_str()).expect("Failed to parse type path");
                    fns_code.push(quote! {
                        #type_path
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
        let should_write = fs::read_to_string(docs.as_path())
            .map(|existing| existing != formatted)
            .unwrap_or(true);
        if should_write {
            if let Some(parent) = docs.parent() {
                fs::create_dir_all(parent).expect("create doc directory error");
            }
            fs::write(docs.as_path(), formatted).expect("create file failed");
        }
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

    fn parse_to_schema(
        &self,
        struct_paths: &mut BTreeSet<String>,
        enum_paths: &mut BTreeSet<String>,
        rs_files: Vec<PathBuf>,
        cache: &mut DocSchemaCache,
        base_path: &Path,
        crate_cache: &mut HashMap<PathBuf, CrateContext>,
    ) {
        for rs_file in rs_files {
            let fingerprint = FileFingerprint::from_path(rs_file.as_path()).ok();
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

            let crate_ctx = resolve_crate_context(rs_file.as_path(), base_path, crate_cache);
            let (structs, enums) = self.extract_schemas(&crate_ctx, rs_file.as_path());
            for value in &structs {
                struct_paths.insert(value.clone());
            }
            for value in &enums {
                enum_paths.insert(value.clone());
            }
            if let Some(fp) = fingerprint {
                cache.update(base_path, rs_file.as_path(), &fp, structs, enums);
            }
        }
    }

    fn extract_schemas(
        &self,
        crate_ctx: &CrateContext,
        rs_file: &Path,
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
        module_stack: &mut Vec<String>,
        structs: &mut Vec<String>,
        enums: &mut Vec<String>,
    ) {
        for item in items {
            match item {
                Item::Struct(item_struct) => {
                    if Self::has_to_schema(&item_struct.attrs) {
                        let path = Self::build_schema_path(
                            crate_path,
                            module_stack,
                            item_struct.ident.to_string(),
                        );
                        structs.push(path);
                    }
                }
                Item::Enum(item_enum) => {
                    if Self::has_to_schema(&item_enum.attrs) {
                        let path = Self::build_schema_path(
                            crate_path,
                            module_stack,
                            item_enum.ident.to_string(),
                        );
                        enums.push(path);
                    }
                }
                Item::Mod(item_mod) => {
                    if let Some((_, nested_items)) = &item_mod.content {
                        module_stack.push(item_mod.ident.to_string());
                        self.collect_items(nested_items, crate_path, module_stack, structs, enums);
                        module_stack.pop();
                    }
                }
                _ => {}
            }
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

    fn is_utoipa_marco(
        &self,
        api_fn: &ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
    ) -> bool {
        api_fn.attrs.as_ref().map_or(false, |attrs| {
            attrs.iter().any(|attr| {
                attr.path()
                    .segments
                    .iter()
                    .any(|segment| segment.ident == "utoipa")
            })
        })
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
