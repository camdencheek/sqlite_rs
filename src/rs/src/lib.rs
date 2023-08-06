#![allow(non_snake_case, unused, non_camel_case_types, non_upper_case_globals)]
#![feature(allocator_api, new_uninit)]

mod agg;
mod auth;
mod autoinc;
mod btree;
mod build;
mod coll_seq;
mod column;
mod cte;
mod date;
mod db;
mod errors;
mod expr;
mod fkey;
mod from;
mod func;
mod global;
mod hash;
mod id;
mod index;
mod lookaside;
mod macros;
mod mem;
mod mem2;
mod module;
mod namecontext;
mod pager;
mod parse;
mod pcache;
mod record;
mod returning;
mod rowset;
mod savepoint;
mod schema;
mod select;
mod table;
mod token;
mod token_type;
mod trigger;
mod upsert;
mod util;
mod vdbe;
mod vtable;
mod whereint;
mod window;
mod with;

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct sqlite3_vtab {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct sqlite3_module {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct sqlite3_context {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct sqlite3_value {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct sqlite3_pcache {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct sqlite3_pcache_page {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

use mem::SQLiteAllocator;

#[global_allocator]
static ALLOCATOR: SQLiteAllocator = SQLiteAllocator();
