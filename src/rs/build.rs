use cbindgen::{Config, EnumConfig, ExportConfig, FunctionConfig, ItemType, Language, RenameRule};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

const custom: &str = r#"
typedef struct ExprList ExprList;
typedef struct With With;
typedef struct FKey FKey;
"#;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = out_dir
        .join(format!("{}.h", package_name))
        .display()
        .to_string();

    let config = Config {
        include_guard: Some("SQLITE3_RS".into()),
        language: Language::C,
        after_includes: Some(custom.into()),
        defines: {
            let mut h = HashMap::new();
            h.insert("debug".into(), "SQLITE_DEBUG".into());
            h.insert("omit_windowfunc".into(), "SQLITE_OMIT_WINDOWFUNC".into());
            h.insert(
                "omit_shared_cache".into(),
                "SQLITE_OMIT_SHARED_CACHE".into(),
            );
            h.insert("coverage_test".into(), "SQLITE_COVERAGE_TEST".into());
            h.insert(
                "omit_progress_callback".into(),
                "SQLITE_OMIT_PROGRESS_CALLBACK".into(),
            );
            h.insert(
                "enable_explain_comments".into(),
                "SQLITE_ENABLE_EXPLAIN_COMMENTS".into(),
            );
            h
        },
        function: FunctionConfig {
            // TODO: figure out why this isn't working
            rename_args: RenameRule::CamelCase,
            ..Default::default()
        },
        enumeration: EnumConfig {
            rename_variants: RenameRule::ScreamingSnakeCase,
            prefix_with_name: true,
            ..Default::default()
        },
        export: ExportConfig {
            include: vec![
                "sqlite3UpperToLower".into(),
                "Column".into(),
                "Coltype".into(),
                "Colflag".into(),
                "SqliteAff".into(),
                "LogEst".into(),
                "CollSeq".into(),
                "CteUse".into(),
                "FuncDestructor".into(),
                "IndexSample".into(),
                "tRowcnt".into(),
                "Savepoint".into(),
                "Token".into(),
                "Cte".into(),
                "Expr".into(),
                "ExprList_item".into(),
                "With".into(),
                "Window".into(),
                "SrcItem".into(),
                "AggInfo".into(),
                "Select".into(),
                "Table".into(),
                "IdList_item".into(),
                "sColMap".into(),
                "Parse".into(),
            ],
            item_types: vec![
                ItemType::Constants,
                ItemType::Globals,
                ItemType::Enums,
                ItemType::Structs,
                ItemType::Unions,
                ItemType::Typedefs,
                ItemType::OpaqueItems,
                ItemType::Functions,
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(&output_file);
}
