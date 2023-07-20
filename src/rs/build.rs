use cbindgen::{Config, EnumConfig, ExportConfig, FunctionConfig, ItemType, Language, RenameRule};
use std::env;
use std::path::PathBuf;

const custom: &str = r#"
typedef struct ExprList ExprList;
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
