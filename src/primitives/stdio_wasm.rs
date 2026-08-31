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
//! Since M4 the write side is provided too, so `println` works: `port/write`
//! appends to `crate::outbuf` and the host drains it. Reading is still stubbed —
//! there is no stdin to block on.
//!
//! `port/write` and `port/flush` are declared exactly as in `stream.rs` but
//! implemented *immediately* rather than by yielding an `IoRequest`. That is
//! forced: wasm32 evaluates through bare `execute` with no scheduler, so a
//! `SIG_IO` yield would have nobody to service it and would hang the call. The
//! declaration still fits, because the native versions already have an immediate
//! `SIG_OK` path of their own — the empty-write short-circuit returns
//! `Value::int(0)` without yielding.

use crate::outbuf::{self, Stream};
use crate::port::{Port, PortKind};
use crate::primitives::def::RegionEffect;
use crate::value::fiber::{SignalBits, SIG_OK};
use crate::signals::Signal;
use crate::value::{Arity, Value};

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

/// Which buffer a port's writes belong in. `None` for anything that is not a
/// standard output stream — on this target there is no other place bytes could
/// go, so writing to one is an error rather than a silent success.
fn stream_of(value: &Value) -> Option<Stream> {
    match value.as_external::<Port>()?.kind() {
        PortKind::Stdout => Some(Stream::Out),
        PortKind::Stderr => Some(Stream::Err),
        _ => None,
    }
}

/// (port/write port data) → int
fn prim_port_write(
    ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    args: &[Value],
) -> (SignalBits, Value) {
    let Some(stream) = stream_of(&args[0]) else {
        return crate::rich_error!(
            ctx,
            "unsupported",
            "port/write: wasm32 can only write to stdout or stderr".to_string(),
        );
    };
    // Same shape as the native primitive's contract: the count returned is the
    // length of the data, never a short write.
    let Some(written) = args[1].with_string(|s| {
        outbuf::push(stream, s);
        s.len()
    }) else {
        return crate::rich_error!(
            ctx,
            "type-error",
            format!("port/write: expected string, got {}", args[1].type_name()),
        );
    };
    (SIG_OK, Value::int(written as i64))
}

/// (port/flush port) → nil
///
/// Nothing to flush: a write has already landed in the buffer by the time this
/// is called. It has to exist and succeed anyway, because `println` calls it
/// after every write.
fn prim_port_flush(
    ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    args: &[Value],
) -> (SignalBits, Value) {
    if stream_of(&args[0]).is_none() {
        return crate::rich_error!(
            ctx,
            "unsupported",
            "port/flush: wasm32 can only flush stdout or stderr".to_string(),
        );
    }
    (SIG_OK, Value::NIL)
}

// Spelled exactly as the native entries in `ports.rs`, defaults included, so the
// two targets cannot drift in signal or arity.
primitive! {
    "port/stdin" => prim_port_stdin { doc: "Return a port for standard input.", category: "port", example: "(port/stdin)", effect: RegionEffect::Fresh, }
    "port/stdout" => prim_port_stdout { doc: "Return a port for standard output.", category: "port", example: "(port/stdout)", effect: RegionEffect::Fresh, }
    "port/stderr" => prim_port_stderr { doc: "Return a port for standard error.", category: "port", example: "(port/stderr)", effect: RegionEffect::Fresh, }
    "port/write" => prim_port_write {
        signal: Signal::io_yields_errors(),
        arity: Arity::AtLeast(2),
        doc: "Write all of data to port, looping over short writes. \
              Returns the number of bytes written, which equals the length \
              of data — a caller never loops on the count. Errors if the \
              fd fails part-way, since an unknown prefix reached the peer.",
        params: &["port", "data"],
        category: "port",
        example: "(port/write (port/stdout) \"hello\")",
        aliases: &["stream/write"],
        effect: RegionEffect::Immediate,
    }
    "port/flush" => prim_port_flush {
        signal: Signal::io_yields_errors(),
        arity: Arity::AtLeast(1),
        doc: "Flush port's write buffer.",
        params: &["port"],
        category: "port",
        example: "(port/flush (port/stdout))",
        aliases: &["stream/flush"],
        effect: RegionEffect::Immediate,
    }
}
