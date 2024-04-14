use syn::{LitBool, LitStr, Token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

pub mod api_key_word {
    syn::custom_keyword!(path);
    syn::custom_keyword!(layers);
    syn::custom_keyword!(group);
    syn::custom_keyword!(api);
    syn::custom_keyword!(open);
    syn::custom_keyword!(path_group);
}

pub struct ApiMacroInfo {
    pub path_token: PathToken,
    pub path_group_token: Option<PathGroupToken>,
    pub layers_token: Option<LayersToken>,
    pub group_token: Option<GroupToken>,
    pub api_token: Option<ApiToken>,
    pub open_token: Option<OpenToken>,
}

impl Parse for ApiMacroInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut path_token = None;
        let mut path_group_token = None;
        let mut layers_token = None;
        let mut group_token = None;
        let mut api_token = None;
        let mut open_token = None;
        // 循环解析输入流，直到到达输入的末尾
        while !input.is_empty() {
            // 使用前瞻来检测接下来是否是 `path` 或 `layers`
            let lookahead = input.lookahead1();
            if lookahead.peek(api_key_word::path) {
                if path_token.is_some() {
                    return Err(input.error("Duplicate 'path' keyword"));
                }
                path_token = Some(input.parse::<PathToken>()?);
            } else if lookahead.peek(api_key_word::path_group) {
                if path_token.is_some() {
                    return Err(input.error("Duplicate 'path_group' keyword"));
                }
                path_group_token = Some(input.parse::<PathGroupToken>()?);
            } else if lookahead.peek(api_key_word::layers) {
                if layers_token.is_some() {
                    return Err(input.error("Duplicate 'layers' keyword"));
                }
                layers_token = Some(input.parse::<LayersToken>()?);
            } else if lookahead.peek(api_key_word::group) {
                if group_token.is_some() {
                    return Err(input.error("Duplicate 'group' keyword"));
                }
                group_token = Some(input.parse::<GroupToken>()?);
            } else if lookahead.peek(api_key_word::api) {
                if api_token.is_some() {
                    return Err(input.error("Duplicate 'api' keyword"));
                }
                api_token = Some(input.parse::<ApiToken>()?);
            } else if lookahead.peek(api_key_word::open) {
                if open_token.is_some() {
                    return Err(input.error("Duplicate 'open' keyword"));
                }
                open_token = Some(input.parse::<OpenToken>()?);
            } else {
                // 否则不处理
            }

            // 可以消耗逗号分隔符，如果有的话；这样也能支持逗号分隔的关键字列表
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        // 确保path已经定义
        if path_token.is_none() {
            return Err(input.error("Missing 'path'"));
        }
        Ok(ApiMacroInfo {
            path_token: path_token.unwrap(),
            path_group_token,
            layers_token,
            group_token,
            api_token,
            open_token,
        })
    }
}

pub struct PathToken {
    pub path_token: api_key_word::path,
    pub eq_token: Token![=],
    pub value_token: LitStr,
}

impl Parse for PathToken {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(PathToken {
            path_token: input.parse::<api_key_word::path>()?,
            eq_token: input.parse()?,
            value_token: input.parse()?,
        })
    }
}

pub struct GroupToken {
    pub group_token: api_key_word::group,
    pub eq_token: Token![=],
    pub value_token: LitStr,
}

impl Parse for GroupToken {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(GroupToken {
            group_token: input.parse::<api_key_word::group>()?,
            eq_token: input.parse()?,
            value_token: input.parse()?,
        })
    }
}

pub struct LayersToken {
    pub layers_token: api_key_word::layers,
    pub eq_token: Token![=],
    pub value_token: Punctuated<LitStr, Token![,]>,
}

impl Parse for LayersToken {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let layers = input.parse::<api_key_word::layers>()?;
        let eq_token = input.parse::<Token![=]>()?;
        let content;
        syn::bracketed!(content in input);
        let value_token = content.parse_terminated(|input: ParseStream| input.parse::<LitStr>(), Token![,])?;

        Ok(LayersToken {
            layers_token: layers,
            eq_token,
            value_token,
        })
    }
}

pub struct ApiToken {
    pub api_token: api_key_word::api,
    pub eq_token: Token![=],
    pub value_token: LitStr,
}

impl Parse for ApiToken {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ApiToken {
            api_token: input.parse::<api_key_word::api>()?,
            eq_token: input.parse()?,
            value_token: input.parse()?,
        })
    }
}

pub struct OpenToken {
    pub open_token: api_key_word::open,
    pub eq_token: Token![=],
    pub value_token: LitBool,
}

impl Parse for OpenToken {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(OpenToken {
            open_token: input.parse::<api_key_word::open>()?,
            eq_token: input.parse()?,
            value_token: input.parse()?,
        })
    }
}

pub struct PathGroupToken {
    pub path_group_token: api_key_word::path_group,
    pub eq_token: Token![=],
    pub value_token: LitStr,
}

impl Parse for PathGroupToken {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(PathGroupToken {
            path_group_token: input.parse::<api_key_word::path_group>()?,
            eq_token: input.parse()?,
            value_token: input.parse()?,
        })
    }
}