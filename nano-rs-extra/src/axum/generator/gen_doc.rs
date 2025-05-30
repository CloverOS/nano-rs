use quote::{quote, ToTokens};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_str, Attribute, FnArg, Item, ItemEnum, ItemMod, ItemStruct, ItemUse, Meta, TypePath,
};
use utoipa::openapi::{
    Contact, ExternalDocs, Info, License, Object, SecurityRequirement, Server, Tag,
};

use nano_rs_build::api_fn::ApiFn;
use nano_rs_build::api_gen::GenDoc;

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
        let mut struct_map: HashMap<String, ItemStruct> = HashMap::new();
        let mut enum_map: HashMap<String, ItemEnum> = HashMap::new();
        self.parse_to_schema(&mut struct_map, &mut enum_map, rs_files);
        let docs = path_buf.join(self.get_doc_file_path());
        if !docs.exists() {
            fs::write(docs.as_path(), "").expect("create routes files error");
        }

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
            let name = tag.clone().name;
            let description = tag.clone().description.unwrap_or("".to_string());
            tags_code.push(quote! {
                (name = #name, description = #description),
            })
        }

        let mut components_code = vec![];
        let mut struct_map_keys: Vec<_> = struct_map.keys().collect();
        struct_map_keys.sort();
        for key in struct_map_keys {
            let type_path: TypePath = parse_str(key.as_str())
                .expect(format!("Failed to parse type path -> {}", key.clone()).as_str());
            components_code.push(quote! {
                #type_path
            });
        }
        let mut enum_map_keys: Vec<_> = enum_map.keys().collect();
        enum_map_keys.sort();
        for key in enum_map_keys {
            let type_path: TypePath = parse_str(key.as_str())
                .expect(format!("Failed to parse type path -> {}", key.clone()).as_str());
            components_code.push(quote! {
                #type_path
            });
        }

        let title = &self.info.title.clone();
        let description = &self.info.description.clone().unwrap_or("".to_string());
        let version = &self.info.version.clone();
        let license_name = &self.info.license.clone().unwrap_or(License::default()).name;
        let license_url = &self
            .info
            .license
            .clone()
            .unwrap_or(License::default())
            .url
            .unwrap_or("".to_string());
        let contact_name = &self
            .info
            .contact
            .clone()
            .unwrap_or(Contact::default())
            .name
            .unwrap_or("".to_string());
        let contact_email = &self
            .info
            .contact
            .clone()
            .unwrap_or(Contact::default())
            .email
            .unwrap_or("".to_string());
        let contact_url = &self
            .info
            .contact
            .clone()
            .unwrap_or(Contact::default())
            .url
            .unwrap_or("".to_string());
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
            let server_url = server.clone().url;
            let server_description = server.clone().description.unwrap_or("".to_string());
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
        let syntax_tree = syn::parse_file(doc_code.to_string().as_str()).unwrap();
        let formatted = prettyplease::unparse(&syntax_tree);
        fs::write(docs.as_path(), formatted).expect("create file failed");
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

pub struct RsFile {
    pub path: PathBuf,
    pub mods: Vec<ItemMod>,
    pub uses: Vec<ItemUse>,
}

impl AxumGenDoc {
    pub fn new() -> AxumGenDocBuilder {
        AxumGenDocBuilder::default()
    }

    fn parse_to_schema(
        &self,
        struct_map: &mut HashMap<String, ItemStruct>,
        enum_map: &mut HashMap<String, ItemEnum>,
        rs_files: Vec<PathBuf>,
    ) {
        for rs_file in rs_files {
            let src = fs::read_to_string(rs_file.clone()).expect("read file error");
            let syntax_tree = syn::parse_file(&src).expect("parse file error");
            for item in syntax_tree.items {
                match item {
                    Item::Struct(item_struct) => {
                        for attr in item_struct.attrs.iter() {
                            if attr.path().is_ident("derive") {
                                if let Meta::List(meta_list) = &attr.meta {
                                    //derive ToSchema
                                    if meta_list
                                        .tokens
                                        .to_token_stream()
                                        .to_string()
                                        .contains("ToSchema")
                                    {
                                        struct_map.insert(
                                            format!(
                                                "{}::{}",
                                                self.parse_path_to_crate(&rs_file),
                                                item_struct.ident.to_string()
                                            ),
                                            item_struct.clone(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Item::Enum(item_enum) => {
                        for attr in item_enum.attrs.iter() {
                            if attr.path().is_ident("derive") {
                                if let Meta::List(meta_list) = &attr.meta {
                                    //derive ToSchema
                                    if meta_list
                                        .tokens
                                        .to_token_stream()
                                        .to_string()
                                        .contains("ToSchema")
                                    {
                                        enum_map.insert(
                                            format!(
                                                "{}::{}",
                                                self.parse_path_to_crate(&rs_file),
                                                item_enum.ident.to_string()
                                            ),
                                            item_enum.clone(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Item::Mod(item_mod) => {
                        self.parse_to_schema_in_mod(
                            struct_map,
                            enum_map,
                            &rs_file,
                            &item_mod,
                            item_mod.ident.to_string(),
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    fn parse_to_schema_in_mod(
        &self,
        struct_map: &mut HashMap<String, ItemStruct>,
        enum_map: &mut HashMap<String, ItemEnum>,
        rs_file: &PathBuf,
        item_mod: &ItemMod,
        mod_name: String,
    ) {
        for content in item_mod.content.iter() {
            for item in content.clone().1.iter() {
                match item {
                    Item::Mod(item_mod) => self.parse_to_schema_in_mod(
                        struct_map,
                        enum_map,
                        rs_file,
                        item_mod,
                        format!("{}::{}", mod_name, item_mod.ident.to_string()),
                    ),
                    Item::Struct(item_struct) => {
                        for attr in item_struct.attrs.iter() {
                            if attr.path().is_ident("derive") {
                                if let Meta::List(meta_list) = &attr.meta {
                                    //derive ToSchema
                                    if meta_list
                                        .tokens
                                        .to_token_stream()
                                        .to_string()
                                        .contains("ToSchema")
                                    {
                                        struct_map.insert(
                                            format!(
                                                "{}::{}::{}",
                                                self.parse_path_to_crate(rs_file),
                                                mod_name,
                                                item_struct.ident.to_string()
                                            ),
                                            item_struct.clone(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Item::Enum(item_enum) => {
                        for attr in item_enum.attrs.iter() {
                            if attr.path().is_ident("derive") {
                                if let Meta::List(meta_list) = &attr.meta {
                                    //derive ToSchema
                                    if meta_list
                                        .tokens
                                        .to_token_stream()
                                        .to_string()
                                        .contains("ToSchema")
                                    {
                                        enum_map.insert(
                                            format!(
                                                "{}::{}::{}",
                                                self.parse_path_to_crate(rs_file),
                                                mod_name,
                                                item_enum.ident.to_string()
                                            ),
                                            item_enum.clone(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn parse_path_to_crate(&self, rs_file: &PathBuf) -> String {
        let path = rs_file.as_path();
        // split src
        if let Some(position) = path.components().position(|comp| comp.as_os_str() == "src") {
            let sub_path = path.components().skip(position + 1);

            //gen crate path
            let mut crate_path = sub_path.fold("crate".to_string(), |mut acc, comp| {
                let comp_str = comp.as_os_str().to_string_lossy();
                acc.push_str("::");
                acc.push_str(&comp_str);
                acc
            });
            // check mod.rs
            crate_path = crate_path.replace("::mod.rs", "");

            // remove .rs
            if let Some(stripped_path) = crate_path.strip_suffix(".rs") {
                stripped_path.to_string()
            } else {
                crate_path
            }
        } else {
            "".to_string()
        }
    }

    fn is_utoipa_marco(
        &self,
        api_fn: &ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
    ) -> bool {
        if let Some(attrs) = &api_fn.attrs {
            for attr in attrs {
                if attr.path().to_token_stream().to_string().contains("utoipa") {
                    return true;
                }
            }
        }
        false
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
