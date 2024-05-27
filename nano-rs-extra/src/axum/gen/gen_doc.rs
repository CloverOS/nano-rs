use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;

use okapi::Map;
use okapi::openapi3::{Components, ExternalDocs, Info, Object, OpenApi, Operation, Parameter, ParameterValue, PathItem, Ref, RefOr, SchemaObject, SecurityRequirement, Server, Tag};
use okapi::schemars::schema::{InstanceType, ObjectValidation, Schema, SingleOrVec};
use quote::{quote, ToTokens};
use regex::Regex;
use syn::{Expr, Field, FnArg, Item, ItemMod, ItemStruct, ItemUse, Lit, Meta, Pat, PathArguments, PathSegment, Type};
use syn::Pat::TupleStruct;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Type::Path;

use nano_rs_build::api_fn::ApiFn;
use nano_rs_build::api_gen::GenDoc;

use crate::axum::gen::AxumGen;

pub struct AxumGenDoc {
    pub info: Info,
    pub servers: Vec<Server>,
    pub security: Vec<SecurityRequirement>,
    pub tags: Vec<Tag>,
    pub external_docs: Option<ExternalDocs>,
    pub extensions: Object,
}


impl GenDoc for AxumGenDoc {
    fn gen_doc(&self, rs_files: Vec<PathBuf>, path_buf: PathBuf, api_fns: HashMap<String, ApiFn<String, Punctuated<FnArg, Comma>,
        Vec<ItemUse>>>) {
        eprintln!("AxumGenRoute gen_doc in {:?}", path_buf);
        let mut struct_map: HashMap<String, ItemStruct> = HashMap::new();
        self.parse_struct_map(&mut struct_map, rs_files);
        let docs = path_buf.join(self.get_doc_file_path());
        if !docs.exists() {
            fs::write(docs.as_path(), "").expect("create routes files error");
        }
        let mut openapi = OpenApi {
            openapi: OpenApi::default_version(),
            info: self.info.clone(),
            servers: self.servers.clone(),
            paths: Default::default(),
            components: Some(Components::default()),
            security: self.security.clone(),
            tags: self.tags.clone(),
            external_docs: self.external_docs.clone(),
            extensions: self.extensions.clone(),
        };
        let mut api_fns_keys: Vec<_> = api_fns.keys().collect();
        api_fns_keys.sort();
        for x in api_fns_keys {
            if let Some(api_fn) = api_fns.get(x) {
                let (path, path_item) = self.gen_path_item(&mut openapi, api_fn, &struct_map);
                openapi.paths.insert(path, path_item);
            }
        }
        let open_api_json = serde_json::to_string(&openapi).expect("openapi to string error");
        let doc_code = quote! {
            pub const DOC_JSON: &'static str = #open_api_json;
        };
        let syntax_tree = syn::parse_file(doc_code.to_string().as_str()).unwrap();
        let formatted = prettyplease::unparse(&syntax_tree);
        fs::write(docs.as_path(), formatted).expect("create file failed");
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

    pub fn gen_path_item(&self, openapi: &mut OpenApi, api_fn: &ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>>, struct_map: &HashMap<String,
        ItemStruct>)
                         -> (String, PathItem) {
        let (path, params) = self.convert_and_extract_params(&api_fn.path);
        let mut path_item = PathItem::default();
        let mut op = Operation::default();
        if let Some(api_doc) = &api_fn.api_fn_doc {
            op.description = Some(api_doc.api.clone());
            op.tags.push(api_doc.group.clone());
        }
        op.servers = Some(self.servers.clone());
        op.summary = Some(api_fn.api_fn_name.clone());
        op.parameters = vec![];
        if let Some(inputs) = api_fn.inputs.clone() {
            for x in inputs.iter() {
                match x {
                    FnArg::Receiver(_) => {}
                    FnArg::Typed(pat_type) => {
                        if let Path(type_path) = *pat_type.ty.clone() {
                            if let Some(segment) = type_path.path.segments.last() {
                                match segment.ident.to_string().as_str() {
                                    "Path" => {
                                        self.parse_path(&mut op, api_fn, struct_map, *pat_type.pat.clone(), segment);
                                    }
                                    "Json" => {
                                        self.parse_json(&mut op, openapi, api_fn, struct_map, segment);
                                    }
                                    "Form" => {}
                                    "Query" => {}
                                    "Header" => {}
                                    _ => {}
                                }
                            }
                        }
                        // eprintln!("pat_type -> {:?}", pat_type.to_token_stream().to_string());
                    }
                }
            }
        }
        match api_fn.method.as_str() {
            "get" => {
                path_item.get = Some(op);
            }
            "post" => {
                path_item.post = Some(op);
            }
            "put" => {
                path_item.put = Some(op);
            }
            "delete" => {
                path_item.delete = Some(op);
            }
            "options" => {
                path_item.options = Some(op);
            }
            "head" => {
                path_item.head = Some(op);
            }
            "patch" => {
                path_item.patch = Some(op);
            }
            "trace" => {
                path_item.trace = Some(op);
            }
            _ => {}
        }
        (path, path_item)
    }

    fn parse_struct_map(&self, struct_map: &mut HashMap<String, ItemStruct>, rs_files: Vec<PathBuf>) {
        for rs_file in rs_files {
            let src = fs::read_to_string(rs_file.clone()).expect("read file error");
            let syntax_tree = syn::parse_file(&src).expect("parse file error");
            for item in syntax_tree.items {
                match item {
                    Item::Struct(item_struct) => {
                        struct_map.insert(format!("{}::{}", self.parse_path_to_crate(&rs_file), item_struct.ident.to_string()), item_struct
                            .clone());
                    }
                    Item::Mod(item_mod) => {
                        self.parse_struct_map_in_mod(struct_map, &rs_file, &item_mod, item_mod.ident.to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    fn parse_struct_map_in_mod(&self, struct_map: &mut HashMap<String, ItemStruct>, rs_file: &PathBuf, item_mod: &ItemMod, mod_name: String) {
        for content in item_mod.content.iter() {
            for item in content.clone().1.iter() {
                match item {
                    Item::Mod(item_mod) => {
                        self.parse_struct_map_in_mod(struct_map, rs_file, item_mod, format!("{}::{}", mod_name, item_mod.ident.to_string()))
                    }
                    Item::Struct(item_struct) => {
                        struct_map.insert(format!("{}::{}::{}", self.parse_path_to_crate(rs_file), mod_name, item_struct.ident.to_string()), item_struct.clone());
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
            let crate_path = sub_path.fold("crate".to_string(), |mut acc, comp| {
                let comp_str = comp.as_os_str().to_string_lossy();
                acc.push_str("::");
                acc.push_str(&comp_str);
                acc
            });

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

    fn parse_json(&self, op: &mut Operation, openapi: &mut OpenApi, api_fn: &ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>>, struct_map:
    &HashMap<String, ItemStruct>, segment: &PathSegment) {
        let path_type = self.extract_params(&segment.arguments);
        let is_primitive_or_string = self.is_primitive_or_string(path_type.as_str());
        if is_primitive_or_string {
            return;
        }
        if let Some(use_crate) = &api_fn.use_crate {
            if let Some(crate_string) = self.get_full_crate_name(path_type.clone(), use_crate) {
                op.request_body = Some(RefOr::Ref(Ref {
                    reference: format!("#/components/schemas/{}", crate_string),
                }));
                self.get_component_ref(openapi, crate_string, struct_map);
            }
        }
    }

    fn parse_path(&self, op: &mut Operation, api_fn: &ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>>, struct_map: &HashMap<String, ItemStruct>, pat: Pat, segment: &PathSegment) {
        let path_type = self.extract_params(&segment.arguments);
        let is_primitive_or_string = self.is_primitive_or_string(path_type.as_str());
        if is_primitive_or_string {
            if let TupleStruct(tuple_struct) = pat {
                if let Some(pat) = tuple_struct.elems.last() {
                    op.parameters.push(self.get_path_params(pat.to_token_stream().to_string(), path_type, None));
                }
            }
        } else {
            // get struct fields
            if let Some(use_crate) = &api_fn.use_crate {
                if let Some(crate_string) = self.get_full_crate_name(path_type.clone(), use_crate) {
                    if let Some(item_struct) = struct_map.get(crate_string.as_str()) {
                        if let syn::Fields::Named(fields) = &item_struct.fields {
                            for field in &fields.named {
                                let (param, path_type, desc) = self.get_path_struct_field_info(field);
                                op.parameters.push(self.get_path_params(param, path_type, desc));
                            }
                        }
                    }
                }
            }
        }
    }

    fn parse_form(&self) {
        //todo
    }

    fn parse_multi_form(&self) {
        //todo
    }

    fn parse_query(&self) {
        //todo
    }

    fn parse_header(&self) {
        //todo
    }

    fn get_component_ref(&self, openapi: &mut OpenApi, struct_name: String, struct_map: &HashMap<String, ItemStruct>) {
        if let Some(component_ref) = &openapi.components {
            let mut component = component_ref.clone();
            if component.schemas.get(struct_name.as_str()).is_some() {
                return;
            }
            let mut schemas = SchemaObject::default();
            schemas.instance_type = Some(SingleOrVec::Single(Box::new(InstanceType::Object)));
            let mut object_validation = ObjectValidation::default();
            let mut properties: Map<String, Schema> = Map::new();
            if let Some(item_struct) = struct_map.get(struct_name.as_str()) {
                self.parse_struct(&mut properties, item_struct, openapi, &struct_name, struct_map);
            }
            object_validation.properties = properties;
            schemas.object = Some(Box::new(object_validation));
            component.schemas.insert(struct_name, schemas);
            openapi.components = Some(component);
        }
    }

    fn get_path_params(&self, param: String, type_name: String, description: Option<String>) -> RefOr<Parameter> {
        RefOr::Object(Parameter {
            name: param.to_string(),
            location: "path".to_string(),
            description,
            required: true,
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Schema {
                style: None,
                explode: None,
                allow_reserved: false,
                schema: SchemaObject {
                    metadata: None,
                    instance_type: Some(SingleOrVec::Single(Box::new(self.get_instance_type(type_name)))),
                    format: None,
                    enum_values: None,
                    const_value: None,
                    subschemas: None,
                    number: None,
                    string: None,
                    array: None,
                    object: None,
                    reference: None,
                    extensions: Default::default(),
                },
                example: None,
                examples: None,
            },
            extensions: Default::default(),
        })
    }

    fn parse_struct(&self, properties: &mut Map<String, Schema>, item_struct: &ItemStruct, openapi: &mut OpenApi, struct_name: &String, struct_map: &HashMap<String, ItemStruct>) {
        if let syn::Fields::Named(fields) = &item_struct.fields {
            for field in &fields.named {
                let mut schema_object = SchemaObject::default();
                if let Some(ident) = &field.ident {
                    match &field.ty {
                        Path(path) => {
                            if let Some(segment) = path.path.segments.last() {
                                let param_type = self.extract_params(&segment.arguments);
                                if param_type.is_empty() {} else {
                                    // process object
                                }
                                eprintln!("typePath ->{:?}", segment.to_token_stream().to_string());
                            }
                        }
                        _ => {}
                    }
                    for attr in &field.attrs {
                        if attr.path().is_ident("doc") {
                            if let Meta::NameValue(name_value) = &attr.meta {
                                if let Expr::Lit(lit) = &name_value.value {
                                    if let Lit::Str(str) = &lit.lit {}
                                }
                            }
                        }
                    }
                }
                properties.insert(struct_name.to_string(), Schema::Object(schema_object));
            }
        }
    }

    fn get_path_struct_field_info(&self, field: &Field) -> (String, String, Option<String>) {
        let mut param = String::new();
        let mut path_type = String::new();
        let mut desc = None;

        if let Some(ident) = &field.ident {
            param = ident.to_string();
            path_type = self.get_type_name(&field.ty);
            for attr in &field.attrs {
                if attr.path().is_ident("doc") {
                    if let Meta::NameValue(name_value) = &attr.meta {
                        if let Expr::Lit(lit) = &name_value.value {
                            if let Lit::Str(str) = &lit.lit {
                                desc = Some(str.value());
                            }
                        }
                    }
                }
            }
        }
        (param, path_type, desc)
    }

    fn extract_params(&self, arguments: &PathArguments) -> String {
        if let PathArguments::AngleBracketed(arg) = arguments {
            arg.args.iter().map(|arg| arg.to_token_stream().to_string()).collect::<Vec<String>>().join(", ")
        } else {
            String::from("")
        }
    }

    fn is_primitive_or_string(&self, type_name: &str) -> bool {
        matches!(type_name, "i32" | "u64" | "f32" | "bool" | "char" | "i8" | "i16" | "i64" | "u8" | "u16" | "u32" | "usize" | "isize" | "f64" |
            "str"|"String")
    }

    fn get_type_name(&self, ty: &Type) -> String {
        match ty {
            Path(type_path) => {
                if let Some(segment) = type_path.path.segments.last() {
                    segment.ident.to_string()
                } else {
                    String::from("Unknown Type")
                }
            }
            _ => String::from("Complex or Unsupported Type")
        }
    }

    fn get_instance_type(&self, type_name: String) -> InstanceType {
        match type_name.as_str() {
            "f32" | "f64" => {
                InstanceType::Number
            }
            "i32" | "u64" | "i8" | "i16" | "i64" | "u8" | "u16" | "u32" | "usize" | "isize" => {
                InstanceType::Integer
            }
            "bool" => {
                InstanceType::Boolean
            }
            "Vec" => {
                InstanceType::Array
            }
            "char" | "str" | "String" => {
                InstanceType::String
            }
            _ => {
                InstanceType::Null
            }
        }
    }

    fn convert_and_extract_params(&self, path: &str) -> (String, Vec<String>) {
        let re = Regex::new(r":(\w+)").unwrap();
        let mut params = Vec::new();

        let new_path = re.replace_all(path, |caps: &regex::Captures| {
            params.push(caps[1].to_string());
            format!("{{{}}}", &caps[1])
        }).to_string();

        (new_path, params)
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