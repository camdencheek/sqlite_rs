// Opaque struct because we do not want Rust to know
// it's a dynamically sized type.
// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
pub struct With {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
