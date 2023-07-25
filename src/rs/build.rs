use cbindgen::{
    Config, EnumConfig, ExportConfig, FunctionConfig, ItemType, Language, MacroExpansionConfig,
    RenameRule,
};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
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
        sort_by: cbindgen::SortKey::Name,
        after_includes: Some(
            r#"
typedef uint16_t TERM;
typedef uint16_t WO;
"#
            .into(),
        ),
        defines: {
            let mut h = HashMap::new();
            h.insert("debug".into(), "SQLITE_DEBUG".into());
            h.insert("omit_windowfunc".into(), "SQLITE_OMIT_WINDOWFUNC".into());
            h.insert("omit_cast".into(), "SQLITE_OMIT_CAST".into());
            h.insert("omit_wal".into(), "SQLITE_OMIT_WAL".into());
            h.insert("enable_stat4".into(), "SQLITE_ENABLE_STAT4".into());
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
            h.insert("small_stack".into(), "SQLITE_SMALL_STACK".into());
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
                "WhereOrSet".into(),
                "WhereMaskSet".into(),
                "WherePath".into(),
                "WhereScan".into(),
                "WhereLoopBuilder".into(),
                "Colflag".into(),
                "CollSeq".into(),
                "Coltype".into(),
                "CellInfo".into(),
                "Column".into(),
                "WHERE".into(),
                "WO".into(),
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
                "VTable".into(),
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
                "VtabCtx".into(),
                "TF".into(),
                "WhereMemBlock".into(),
                "WhereRightJoin".into(),
                "WhereLevel".into(),
                "InLoop".into(),
                "WhereLoop".into(),
                "WhereTerm".into(),
                "WhereClause".into(),
                "WhereMaskSet".into(),
                "TERM".into(),
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

    let mut buf = Vec::new();
    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write(&mut buf);
    buf = reorder(buf);

    let mut f = File::create(&output_file).unwrap();
    f.write_all(&buf).unwrap();
}

// HACK: reorder the struct definitions along with some pre-declarations
// so the header can actually build
fn reorder(input: Vec<u8>) -> Vec<u8> {
    use regex::bytes::Regex;
    let (input, where_clause) = extract(input, "WhereClause");
    let (input, where_term) = extract(input, "WhereTerm");
    let (input, where_or_info) = extract(input, "WhereOrInfo");
    let (input, where_and_info) = extract(input, "WhereAndInfo");
    let (input, where_term_u_x) = extract(input, "WhereTerm_u_x");
    let (mut input, where_term_u) = extract(input, "WhereTerm_u");

    let dst = Regex::new(r"(?s)typedef union WhereLoop_u \{.*\} WhereLoop_u;\n")
        .unwrap()
        .find(&input)
        .unwrap()
        .end();

    let where_clause_decl: Vec<u8> = b"typedef struct WhereClause WhereClause;\n".to_vec();
    let where_or_info_decl: Vec<u8> = b"typedef struct WhereOrInfo WhereOrInfo;\n".to_vec();
    let where_and_info_decl: Vec<u8> = b"typedef struct WhereAndInfo WhereAndInfo;\n".to_vec();

    let _ = input
        .splice(
            dst..dst,
            where_clause_decl
                .into_iter()
                .chain(where_or_info_decl)
                .chain(where_and_info_decl)
                .chain(where_term_u_x)
                .chain(where_term_u)
                .chain(where_term)
                .chain(where_clause)
                .chain(where_and_info)
                .chain(where_or_info),
        )
        .collect::<Vec<_>>();
    input
}

fn extract(mut input: Vec<u8>, target: &str) -> (Vec<u8>, Vec<u8>) {
    use regex::bytes::Regex;
    let restr = format!(
        r"(?s)typedef (struct|union) {} \{{.*\}} {};\n",
        &target, &target
    );
    dbg!(&restr);
    let re = Regex::new(&restr).unwrap();
    let m = re.find(&input).unwrap();
    let copy = input[m.range()].to_owned();
    input.splice(m.range(), []);
    (input, copy)
}
