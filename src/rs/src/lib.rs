mod hash;
mod mem;
mod util;

pub use hash::*;
use mem::SQLiteAllocator;
pub use util::strings::*;

#[global_allocator]
static ALLOCATOR: SQLiteAllocator = SQLiteAllocator();
