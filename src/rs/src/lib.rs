mod agg;
mod coll_seq;
mod column;
mod cte;
mod expr;
mod fkey;
mod from;
mod func;
mod hash;
mod id;
mod index;
mod mem;
mod module;
mod parse;
mod savepoint;
mod schema;
mod select;
mod src;
mod table;
mod token;
mod trigger;
mod upsert;
mod util;
mod vtable;
mod window;
mod with;

pub struct sqlite3;
pub struct sqlite3_vtab;
pub struct sqlite3_module;
pub struct sqlite3_context;
pub struct sqlite3_value;

use mem::SQLiteAllocator;

#[global_allocator]
static ALLOCATOR: SQLiteAllocator = SQLiteAllocator();
