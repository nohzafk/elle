//! The three stdio port constructors, for wasm32.
//!
//! These are *not* stand-ins. `mod ports` is compiled out on wasm32 because its
//! other primitives build `IoRequest`s, but these three only mint a `Port` value
//! — `Port::stdin/stdout/stderr` carry `fd: None` and touch no descriptor, which
//! is precisely why `mod port` was kept alive on this target. So they are
//! provided for real and excluded from `stub_wasm` (see `tools/gen-wasm-stubs.sh`,
//! which fails if a name listed there stops existing).
//!
//! They have to work, not merely be bound, because stdlib.lisp calls them at
//! *load* time:
//!
//! ```text
//! (def *stdin*  (parameter (port/stdin)))
//! (def *stdout* (parameter (port/stdout)))
//! (def *stderr* (parameter (port/stderr)))
//! ```
//!
//! Those are top-level forms, so a stub answering `:unsupported` aborts
//! `init_stdlib` itself — no stdlib, and therefore no arithmetic, no `let`, no
//! Elle at all on wasm32. A bound-but-failing name is not enough here.
//!
//! What a port cannot yet do on this target is carry bytes: `port/write` and
//! friends stay stubbed until an embedder supplies a console callback. So
//! `*stdout*` is a real port that `println` cannot write to yet — the value
//! exists, the transport does not.

use crate::port::Port;
use crate::primitives::def::RegionEffect;
use crate::value::fiber::{SignalBits, SIG_OK};
use crate::value::Value;

/// (port/stdin) → port
fn prim_port_stdin(
    ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    _args: &[Value],
) -> (SignalBits, Value) {
    (SIG_OK, ctx.external("port", Port::stdin()))
}

/// (port/stdout) → port
fn prim_port_stdout(
    ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    _args: &[Value],
) -> (SignalBits, Value) {
    (SIG_OK, ctx.external("port", Port::stdout()))
}

/// (port/stderr) → port
fn prim_port_stderr(
    ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    _args: &[Value],
) -> (SignalBits, Value) {
    (SIG_OK, ctx.external("port", Port::stderr()))
}

// Spelled exactly as the native entries in `ports.rs`, defaults included, so the
// two targets cannot drift in signal or arity.
primitive! {
    "port/stdin" => prim_port_stdin { doc: "Return a port for standard input.", category: "port", example: "(port/stdin)", effect: RegionEffect::Fresh, }
    "port/stdout" => prim_port_stdout { doc: "Return a port for standard output.", category: "port", example: "(port/stdout)", effect: RegionEffect::Fresh, }
    "port/stderr" => prim_port_stderr { doc: "Return a port for standard error.", category: "port", example: "(port/stderr)", effect: RegionEffect::Fresh, }
}
