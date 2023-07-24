use cbindgen::{
    Config, EnumConfig, ExportConfig, FunctionConfig, ItemType, Language, MacroExpansionConfig,
    RenameRule,
};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

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
        macro_expansion: MacroExpansionConfig { bitflags: true },
        defines: {
            let mut h = HashMap::new();
            h.insert("debug".into(), "SQLITE_DEBUG".into());
            h.insert("omit_windowfunc".into(), "SQLITE_OMIT_WINDOWFUNC".into());
            h.insert("omit_cast".into(), "SQLITE_OMIT_CAST".into());
            h.insert("omit_wal".into(), "SQLITE_OMIT_WAL".into());
            h.insert(
                "user_authentication".into(),
                "SQLITE_USER_AUTHENTICATION".into(),
            );
            h.insert(
                "enable_unlock_notify".into(),
                "SQLITE_ENABLE_UNLOCK_NOTIFY".into(),
            );
            h.insert(
                "omit_virtualtable".into(),
                "SQLITE_OMIT_VIRTUALTABLE".into(),
            );
            h.insert(
                "omit_progress_callback".into(),
                "SQLITE_OMIT_PROGRESS_CALLBACK".into(),
            );
            h.insert(
                "omit_authorization".into(),
                "SQLITE_OMIT_AUTHORIZATION".into(),
            );
            h.insert(
                "omit_shared_cache".into(),
                "SQLITE_OMIT_SHARED_CACHE".into(),
            );
            h.insert("omit_deprecated".into(), "SQLITE_OMITE_DEPRECATED".into());
            h.insert(
                "enable_preupdate_hook".into(),
                "SQLITE_ENABLE_PREUPDATE_HOOK".into(),
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
            h.insert(
                "omit_floating_point".into(),
                "SQLITE_OMIT_FLOATING_POINT".into(),
            );
            h.insert("check_pages".into(), "SQLITE_CHECK_PAGES".into());
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
        documentation: false,
        export: ExportConfig {
            include: vec![
                "AggInfo".into(),
                "Colflag".into(),
                "CollSeq".into(),
                "Coltype".into(),
                "CellInfo".into(),
                "Column".into(),
                "Cte".into(),
                "CteUse".into(),
                "Expr".into(),
                "MemPage".into(),
                "sColMap".into(),
                "ExprList_item".into(),
                "FuncDestructor".into(),
                "IdList_item".into(),
                "IndexSample".into(),
                "LogEst".into(),
                "PCache".into(),
                "Parse".into(),
                "Savepoint".into(),
                "Select".into(),
                "SqliteAff".into(),
                "SrcItem".into(),
                "Table".into(),
                "Token".into(),
                "Window".into(),
                "sColMap".into(),
                "sqlite3UpperToLower".into(),
                "tRowcnt".into(),
                "BtLock".into(),
                "BtreePayload".into(),
                "sqlite3CtypeMap".into(),
                "TF".into(),
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
