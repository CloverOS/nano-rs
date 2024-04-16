use std::error::Error;
use std::fs;
use std::path::PathBuf;

use sea_orm::{Database, DatabaseConnection, DbBackend, FromQueryResult, JsonValue, Statement};
use syn::{Item, parse_quote};

use nano_rs_build::api_file::get_rs_files;

pub struct GenComments {
    #[allow(dead_code)]
    gen_path: PathBuf,
    database_url: String,
    database: String,
    schema: String,
    rs_files: Vec<PathBuf>,
}

impl GenComments {
    pub fn new(path: Option<PathBuf>, database_url: String, database: String, schema: Option<String>) -> Self {
        let path_buf;
        if let Some(p) = path {
            path_buf = p;
        } else {
            path_buf = std::env::current_dir().expect("get current dir error");
        }
        let mut rs_files = Vec::new();
        get_rs_files(&mut rs_files, path_buf.as_path()).expect("get rs files error");
        GenComments {
            gen_path: path_buf,
            database_url,
            database,
            schema: schema.unwrap_or_else(|| "public".to_string()),
            rs_files,
        }
    }

    pub async fn gen_comments(&self) -> Result<(), Box<dyn Error>> {
        eprintln!("gen comments");
        let db: DatabaseConnection = Database::connect(self.database_url.clone()).await?;
        let rs_files = self.rs_files.clone();
        for file in rs_files {
            let src = fs::read_to_string(file.clone())?;
            eprintln!("parsing: {:?}", file.clone());
            let mut syntax_tree = syn::parse_file(&src)?;
            for item in &mut syntax_tree.items {
                match item {
                    Item::Struct(ref mut item_struct) => {
                        for attr in &item_struct.clone().attrs {
                            if attr.meta.path().is_ident("sea_orm") {
                                if let Ok(meta_list) = attr.meta.require_list() {
                                    for token in meta_list.clone().tokens {
                                        if let proc_macro2::TokenTree::Literal(lit) = token {
                                            let value = lit.to_string();
                                            let table_name = value.trim_matches('"');
                                            let ret: Vec<JsonValue> = JsonValue::find_by_statement(Statement::from_sql_and_values(
                                                DbBackend::Postgres,
                                                r#"SELECT
                                                            cols.column_name, (
                                                                SELECT
                                                                    pg_catalog.col_description(c.oid, cols.ordinal_position::int)
                                                                FROM
                                                                    pg_catalog.pg_class c
                                                                WHERE
                                                                    c.oid = (SELECT ('"' || cols.table_name || '"')::regclass::oid)
                                                                    AND c.relname = cols.table_name
                                                            ) AS column_comment
                                                        FROM
                                                            information_schema.columns cols
                                                        WHERE
                                                            cols.table_catalog    = $1
                                                            AND cols.table_name   = $2
                                                            AND cols.table_schema = $3;"#,
                                                [self.database.clone().into(), table_name.to_string().into(), self.schema.clone().into()],
                                            ))
                                                .all(&db)
                                                .await?;

                                            for field in &mut item_struct.fields {
                                                for x in &ret {
                                                    if let Some(col_name) = x.get("column_name") {
                                                        if let Some(name_string) = col_name.as_str() {
                                                            if let Some(ident) = field.clone().ident {
                                                                if ident.eq(name_string) {
                                                                    if let Some(comment) = x.get("column_comment") {
                                                                        if let Some(comment_str) = comment.as_str() {
                                                                            let formatted_comment = format!(" {}", comment_str);
                                                                            if field.clone().attrs.len() > 0 {
                                                                                if let Some(attr) = field.clone().attrs.get(0) {
                                                                                    if !attr.meta.path().is_ident("doc") {
                                                                                        field.attrs.push(parse_quote!(
                                                                                            #[doc = #formatted_comment]
                                                                                        ));
                                                                                    }
                                                                                }
                                                                            } else {
                                                                                field.attrs.push(parse_quote!(
                                                                                        #[doc = #formatted_comment]
                                                                                    ));
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            let formatted = prettyplease::unparse(&syntax_tree);
            fs::write(file.as_path(), formatted).expect("create file failed");
        }
        Ok(())
    }
}