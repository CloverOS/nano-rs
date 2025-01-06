#[cfg(feature = "utoipa_axum")]
use crate::axum::gen::parse_utoipa_info;
use crate::axum::gen::AxumGen;
use nano_rs_build::api_fn::ApiFn;
use nano_rs_build::api_gen::GenRoute;
use quote::__private::{Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_quote, parse_str, Attribute, Expr, ExprPath, FnArg, GenericArgument, Ident, ItemUse,
    PathArguments, PathSegment, Type, TypePath,
};

const STATE: &str = "State";

const WITH_LAYER: &str = "#with_layer_";

const WITHOUT_STATE: &str = "without_state";

pub struct AxumGenRoute {}

impl GenRoute for AxumGenRoute {
    fn gen_route(
        &self,
        _rs_files: Vec<PathBuf>,
        path_buf: PathBuf,
        api_fns: HashMap<
            String,
            ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
        >,
    ) {
        eprintln!("AxumGenRoute gen_route in {:?}", path_buf);
        let routes = path_buf.join(self.get_routes_file_path());
        if !routes.exists() {
            fs::write(routes.as_path(), "").expect("create routes files error");
        }
        let mut fn_route_code: HashMap<String, Vec<TokenStream>> = HashMap::new();
        let mut use_crate: HashMap<String, bool> = HashMap::new();

        self.parse_routes(api_fns, &mut fn_route_code, &mut use_crate);

        let mut use_code = self.gen_must_use_code();

        let mut use_crate_sort_key: Vec<_> = use_crate.keys().collect();
        use_crate_sort_key.sort();
        for key in use_crate_sort_key {
            let k = key;
            let item_use: ItemUse =
                syn::parse_str(k).expect("Unable to parse the use crate code string");
            use_code = parse_quote!(
                #use_code
                #item_use
            );
        }
        let mut routes_code = quote!();
        let mut routes_params_code: HashMap<String, TokenStream> = HashMap::new();
        let mut routes_fn_params_code: Vec<TokenStream> = vec![];
        let mut keys: Vec<_> = fn_route_code.keys().collect();
        keys.sort();
        for key in keys {
            let state_raw = key.clone();
            let code = fn_route_code.get(key).expect("Failed to get fn_route_code");
            let mut layer_states_params_map: HashMap<String, TokenStream> = HashMap::new();

            let mut state = state_raw.clone();
            let state_type = state_raw.split("#").next().unwrap_or("");

            let mut layers_code = quote!();
            let mut edit_state = state.clone();

            let state_ident_str = self.camel_to_snake(state_type.replace("::", "_").as_str());
            let ident = Ident::new(state_ident_str.as_str(), Span::call_site());

            for layer_crate in state.split(WITH_LAYER).skip(1).into_iter() {
                let layer_state = self
                    .extract_content(&layer_crate, "#{", "}")
                    .unwrap_or_default();
                let mut layer_string = layer_crate.to_string().clone();
                if layer_string.is_empty() {
                    continue;
                }
                if !layer_state.is_empty() {
                    edit_state = state.replace("#{", "_").replace("}", "");
                    let state = format!("#{{{}}}", layer_state);
                    layer_string = layer_string.replace(state.as_str(), "");
                    let layer: ExprPath =
                        parse_str(layer_string.as_str()).expect("Failed to parse path");
                    if !layer_state.eq(state_type) {
                        let layer_state_type_path: TypePath = parse_str(layer_state.as_str())
                            .expect("Failed to parse state type path");
                        let ident_str =
                            self.camel_to_snake(layer_state.replace("::", "_").as_str());
                        let layer_ident = Ident::new(ident_str.as_str(), Span::call_site());
                        layer_states_params_map
                            .insert(ident_str.clone(), quote!(#layer_state_type_path));
                        layers_code = parse_quote!(
                            #layers_code
                            .route_layer(axum::middleware::from_fn_with_state(
                                #layer_ident,#layer))
                        );
                        routes_params_code
                            .insert(ident_str.clone(), quote!(#layer_state_type_path));
                    } else {
                        layers_code = parse_quote!(
                            #layers_code
                            .route_layer(axum::middleware::from_fn_with_state(
                                #ident.clone(),#layer))
                        );
                        let type_path: TypePath =
                            parse_str(state_type).expect("Failed to parse state type path");
                        routes_params_code.insert(state_ident_str.clone(), quote!(#type_path));
                    }
                } else {
                    if let Ok(layer) = parse_str::<ExprPath>(layer_string.as_str()) {
                        layers_code = parse_quote!(
                            #layers_code
                            .route_layer(axum::middleware::from_fn(#layer))
                        );
                    } else {
                        let layer =
                            parse_str::<Expr>(layer_string.as_str()).expect("layer parse failed");
                        layers_code = parse_quote!(
                            #layers_code
                            .layer(#layer)
                        );
                    }
                }
            }
            state = edit_state;
            state = state
                .replace("#", "_")
                .replace("::", "_")
                .replace("(", "")
                .replace(")", "")
                .replace(".", "_");
            state = self.camel_to_snake(&state);
            let mut let_routes_code = quote!();
            let mut code_sort = code.clone();
            code_sort.sort_by(|a, b| {
                let a_string = a.to_string();
                let b_string = b.to_string();
                a_string.cmp(&b_string)
            });
            for x in code_sort {
                let_routes_code = parse_quote! {
                    #let_routes_code
                    #x
                };
            }
            state = format!("get_routes_{}", state);
            let ident_fn_name = Ident::new(state.as_str(), Span::call_site());
            let mut layer_states_params_tokens = vec![];
            let mut layer_states_params_ident = vec![];
            for (k, v) in layer_states_params_map.iter() {
                let ident = Ident::new(&k, Span::call_site());
                layer_states_params_tokens.push(quote!(#ident: #v));
                layer_states_params_ident.push(quote!(#ident));
            }
            let layer_states_params = quote! {
                #(#layer_states_params_tokens),*
            };
            //with state
            if !state_raw.starts_with(WITHOUT_STATE) {
                let type_path: TypePath =
                    parse_str(state_type).expect("Failed to parse state type path");
                routes_code = parse_quote!(
                    #routes_code

                    pub fn #ident_fn_name(#ident : #type_path,#layer_states_params) -> Router {
                        Router::new()
                            #let_routes_code
                            #layers_code
                            .with_state(#ident)
                    }
                );
                routes_fn_params_code.push(quote!(
                    .merge(#ident_fn_name(#ident.clone(),#(#layer_states_params_ident.clone())*))
                ));
                routes_params_code.insert(state_ident_str.clone(), quote!(#type_path));
            } else {
                //without state
                routes_code = parse_quote!(
                    #routes_code

                    pub fn #ident_fn_name(#layer_states_params) -> Router {
                        Router::new()
                            #let_routes_code
                            #layers_code
                    }
                );
                routes_fn_params_code.push(quote!(
                    .merge(#ident_fn_name(#(#layer_states_params_ident.clone())*))
                ));
            }
        }
        let mut all_routes_params = vec![];
        let mut keys: Vec<_> = routes_params_code.keys().collect();
        keys.sort();
        for key in keys {
            let k = key.clone();
            let v = routes_params_code
                .get(key)
                .expect("Failed to get routes_params_code");
            let ident = Ident::new(&k, Span::call_site());
            all_routes_params.push(quote!(#ident: #v));
        }
        let all_routes_code = quote!(
            pub fn get_routes(#(#all_routes_params),*) -> Router{
                Router::new()
                #(#routes_fn_params_code)*
            }
        );

        let complete_code: TokenStream = parse_quote!(
            #use_code

            #all_routes_code

            #routes_code
        );
        let syntax_tree = syn::parse_file(complete_code.to_string().as_str()).unwrap();
        let formatted = prettyplease::unparse(&syntax_tree);
        fs::write(routes.as_path(), formatted).expect("create file failed");
        let output = Command::new("rustfmt")
            .arg(routes.as_path())
            .output()
            .expect("Failed to execute rustfmt");
        if !output.status.success() {
            eprintln!(
                "Rustfmt failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

impl AxumGenRoute {
    fn gen_must_use_code(&self) -> TokenStream {
        quote!(
            /// Code generated by nano-rs. DO NOT EDIT.
            use axum::Router;
        )
    }

    fn extract_content(&self, s: &str, start_delim: &str, end_delim: &str) -> Option<String> {
        s.split(start_delim)
            .nth(1)
            .and_then(|part| part.split(end_delim).next())
            .map(|s| s.to_string())
    }

    fn camel_to_snake(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c.is_uppercase() {
                // Check if the result is empty and the last character is not an underscore
                if !result.is_empty() && !result.ends_with('_') {
                    result.push('_');
                }
                result.push(c.to_lowercase().next().unwrap());
            } else {
                result.push(c);
            }
        }

        result
    }

    // Handling Generic Parameters of State Type
    fn get_state_type_vec(&self, segment: &PathSegment, mut tp_vec: Vec<String>) -> Vec<String> {
        if let PathArguments::AngleBracketed(angle_bracketed_param) = &segment.arguments {
            for arg in &angle_bracketed_param.args {
                match arg {
                    GenericArgument::Type(Type::Path(arg_type_path)) => {
                        if let Some(ident) = arg_type_path.path.get_ident() {
                            tp_vec.push(ident.to_string());
                        } else {
                            for inner_segment in &arg_type_path.path.segments {
                                tp_vec = self.get_state_type_vec(inner_segment, tp_vec);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        tp_vec
    }

    fn parse_routes(
        &self,
        mut api_fns: HashMap<
            String,
            ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
        >,
        fn_route_code: &mut HashMap<String, Vec<TokenStream>>,
        use_crate_map: &mut HashMap<String, bool>,
    ) {
        for (name, api_fn) in api_fns.iter_mut() {
            #[cfg(feature = "utoipa_axum")]
            parse_utoipa_info(api_fn);
            let path = api_fn.clone().path;
            if let Some(inputs) = api_fn.inputs.clone() {
                if inputs.is_empty() {
                    let key = self.get_fn_code_key(api_fn, None);
                    self.insert_all_route(fn_route_code, use_crate_map, name, api_fn, &path, key);
                    continue;
                }
                let mut with_state = false;
                for arg in inputs.iter() {
                    match arg {
                        FnArg::Typed(pat_type) => {
                            match &*pat_type.ty {
                                Type::Path(type_path) => {
                                    for segment in &type_path.path.segments {
                                        match segment.ident.to_string().as_str() {
                                            //with state
                                            STATE => {
                                                with_state = true;
                                                let tp_vec =
                                                    self.get_state_type_vec(&segment, vec![]);
                                                for x in tp_vec {
                                                    if let Some(use_crate) =
                                                        api_fn.use_crate.clone()
                                                    {
                                                        if let Some(use_string) =
                                                            self.get_full_crate_name(x, &use_crate)
                                                        {
                                                            let key = self.get_fn_code_key(
                                                                api_fn,
                                                                Some(use_string),
                                                            );
                                                            self.insert_all_route(
                                                                fn_route_code,
                                                                use_crate_map,
                                                                name,
                                                                api_fn,
                                                                &path,
                                                                key,
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                            //without state but inputs
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                if !with_state {
                    let key = self.get_fn_code_key(api_fn, None);
                    self.insert_all_route(fn_route_code, use_crate_map, name, api_fn, &path, key);
                }
            }
        }
    }
    fn insert_all_route(
        &self,
        fn_route_code: &mut HashMap<String, Vec<TokenStream>>,
        use_crate_map: &mut HashMap<String, bool>,
        name: &String,
        api_fn: &ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
        path: &String,
        key: String,
    ) {
        const METHODS: [&str; 8] = [
            "post", "get", "put", "delete", "patch", "options", "head", "trace",
        ];
        if METHODS.contains(&api_fn.method.as_str()) {
            eprintln!("method --->{}", api_fn.method.clone());
            self.method_insert(
                api_fn.method.clone(),
                fn_route_code,
                use_crate_map,
                name.clone(),
                path.clone(),
                key,
            );
        }
    }

    fn method_insert(
        &self,
        method: String,
        fn_with_state: &mut HashMap<String, Vec<TokenStream>>,
        use_crate: &mut HashMap<String, bool>,
        name: String,
        path: String,
        key: String,
    ) {
        use_crate.insert(format!("use axum::routing::{};", method), true);
        let ident_fn_name: ExprPath = parse_str(name.as_str()).expect("Failed to parse path");
        let fn_name = Ident::new(method.as_str(), Span::call_site());
        if let Some(v) = fn_with_state.get_mut(&key) {
            v.push(quote!(
                .route(#path,#fn_name(#ident_fn_name))
            ));
        } else {
            fn_with_state.insert(
                key,
                vec![quote!(
                    .route(#path,#fn_name(#ident_fn_name))
                )],
            );
        }
    }
    fn get_fn_code_key(
        &self,
        api_fn: &ApiFn<String, Punctuated<FnArg, Comma>, Vec<ItemUse>, Vec<Attribute>>,
        use_string: Option<String>,
    ) -> String {
        let mut key;
        if let Some(use_string) = use_string {
            key = format!("{}", use_string.clone());
        } else {
            key = WITHOUT_STATE.to_string();
        }
        if api_fn.layers_fn_name.is_some() {
            let mut layers = "".to_string();
            for x in api_fn.layers_fn_name.clone().unwrap().iter() {
                layers = format!("{layers}{WITH_LAYER}{}", x);
            }
            key = format!("{}{}", key, layers);
        }
        key
    }
}

impl AxumGenRoute {
    pub fn new() -> Self {
        AxumGenRoute {}
    }
}

impl AxumGen for AxumGenRoute {}
