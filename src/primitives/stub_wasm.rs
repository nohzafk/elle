//! wasm32 stand-ins for the primitives whose implementing modules are
//! compiled out on that target (`chan`, `concurrency`, `io`, `net`, `unix`,
//! `ports`, `posix`, `stream`, `subprocess`, `watch` — see `lib.rs` on
//! `mod io`).
//!
//! Only the *names* matter here. stdlib.lisp defines wrappers such as
//! `ev/spawn`, `chan/select` and `subprocess/system` whose bodies reference
//! these primitives, and an unbound name is a compile error in Elle — so
//! dropping the names would mean editing the Lisp layer. Instead every name
//! stays bound to a primitive that reports `:unsupported`, the whole Lisp
//! layer compiles unchanged, and a program that actually calls one gets an
//! error it can catch.
//!
//! Aliases are flattened into ordinary entries: what has to match is the set
//! of registered symbols, not which spelling is canonical.
//!
//! Three names from `ports` are deliberately absent — `port/stdin`,
//! `port/stdout` and `port/stderr` are implemented for real in `stdio_wasm`,
//! because stdlib.lisp calls them at load time and a `:unsupported` answer
//! would abort `init_stdlib`. See `PROVIDED` in the generator.
//!
//! GENERATED FILE — do not edit. Run `tools/gen-wasm-stubs.sh` after adding
//! a primitive to any of those modules.

use crate::signals::Signal;
use crate::value::fiber::{SignalBits, SIG_ERROR};
use crate::value::types::Arity;
use crate::value::Value;

/// Every stubbed name shares this implementation. Arity is `AtLeast(0)` on
/// purpose: a caller should be told the operation is unavailable, not that it
/// passed the wrong number of arguments to something that cannot run anyway.
fn prim_unsupported(
    ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    _args: &[Value],
) -> (SignalBits, Value) {
    (
        SIG_ERROR,
        ctx.error(
            "unsupported",
            "this primitive needs OS facilities that wasm32 does not provide",
        ),
    )
}

primitive! {
    "chan" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "chan/clone" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "chan/close" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "chan/close-recv" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "chan/new" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "chan/recv" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "chan/send" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "chan/try-select" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "chan/wait-ready" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "current-thread-id" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "ev/poll-fd" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "ev/sleep" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "exit" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "halt" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "io-backend?" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "io-request?" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "io/backend" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "io/cancel" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "io/reap" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "io/submit" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "io/wait" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "io/workers" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/exit" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/halt" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/sig-close" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/sig-mask" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/sig-next" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/sig-pending" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/sig-raise" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/sig-send" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/sig-watch" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/sig-watching" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/spawn" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/spawn-vm" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/thread-id" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "os/thread-state" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port?" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port/close" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port/encoding" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port/open" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port/open-bytes" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port/open?" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port/path" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port/read" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port/read-all" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port/read-exact" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port/read-line" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port/seek" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port/set-options" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "port/tell" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "stream/read" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "subprocess/exec" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "subprocess/kill" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "subprocess/pid" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "subprocess/wait" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "sys/args" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "sys/argv" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "sys/env" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "sys/exit" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "sys/halt" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "sys/ip?" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "sys/pid" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "sys/resolve" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "sys/spawn" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "sys/spawn-vm" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "sys/thread-id" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "sys/thread-state" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "sys/trap-exit!" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "tcp/accept" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "tcp/connect-ip" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "tcp/listen" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "tcp/shutdown" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "udp/bind" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "udp/recv-from" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "udp/send-to" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "unix/accept" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "unix/connect" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "unix/listen" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "unix/shutdown" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "watch" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "watch-add" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "watch-close" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "watch-next" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
    "watch-remove" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }
}
