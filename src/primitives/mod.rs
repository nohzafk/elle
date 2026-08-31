#[macro_use]
pub mod arg;
#[macro_use]
pub mod def;

pub mod access;
pub mod allocator;
pub mod arena;
pub mod arithmetic;
pub mod array;
pub mod bitwise;
pub mod r#box;
pub mod bytes;
/// Unlike its OS-facing neighbours below, this stays compiled on wasm32:
/// `value::send` and `value::heap` need `SendableValue` and `WakeList` to
/// move channel endpoints between fibers. Inside it, the fd machinery and
/// the primitive table are cfg'd out, so `chan::PRIMITIVES` does not exist
/// there and `stub_wasm` supplies the names instead.
pub mod chan;
pub mod collection;

pub mod comparison;
pub mod compile;
#[cfg(not(target_arch = "wasm32"))]
pub mod concurrency;
pub mod config;
pub mod convert;
pub mod ctx;
pub mod debug;
pub mod disassembly;
pub mod display;
pub mod docs;
pub mod ffi;
pub mod fiber_introspect;
pub mod fibers;
pub mod fileio;
pub mod format;
pub mod formatspec;
pub mod intrinsics;
pub mod introspection;
#[cfg(not(target_arch = "wasm32"))]
pub mod io;
pub mod json;
pub mod kwarg;
pub mod list;
pub mod loading;
pub mod logic;
pub mod lstruct;
pub mod math;
pub mod memory;
pub mod meta;
pub mod module_init;
pub mod modules;
#[cfg(not(target_arch = "wasm32"))]
pub mod net;
pub mod package;
pub mod parameters;
pub mod path;
#[cfg(not(target_arch = "wasm32"))]
pub mod ports;
#[cfg(not(target_arch = "wasm32"))]
pub mod posix;
pub mod read;
pub mod registration;
pub mod seq;
pub mod sets;
pub mod sort;
#[cfg(target_arch = "wasm32")]
pub mod stub_wasm;
pub mod stream;
pub mod string;
pub mod structs;
#[cfg(not(target_arch = "wasm32"))]
pub mod subprocess;
pub mod time;
pub mod traitregistry;
pub mod traits;
pub mod types;
#[cfg(not(target_arch = "wasm32"))]
pub mod unix;
#[cfg(not(target_arch = "wasm32"))]
pub mod watch;
pub use def::{PrimitiveDef, PrimitiveMeta};
pub use docs::help_text;
pub use module_init::init_stdlib;
pub use registration::{
    build_primitive_meta, intern_primitive_names, prim_def, prim_id_of, prim_table_snapshot,
    register_primitives,
};
