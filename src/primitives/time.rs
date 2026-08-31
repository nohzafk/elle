use crate::primitives::def::RegionEffect;
use crate::signals::Signal;
use crate::value::fiber::{SignalBits, SIG_ERROR};
// On wasm32 every primitive in this file refuses, so nothing here ever succeeds
// and `SIG_OK` genuinely has no use — cfg'd out rather than allow'd, so that a
// future primitive that *can* answer on that target fails to compile until it
// brings the import back.
#[cfg(not(target_arch = "wasm32"))]
use crate::value::fiber::SIG_OK;
use crate::value::types::Arity;
use crate::value::Value;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

// wasm32-unknown-unknown has no clock and no threads at all: `Instant::now`,
// `SystemTime::now` and `thread::sleep` are all present as *compiling* stubs
// that panic when called (std's `sys/time/unsupported.rs` and
// `sys/thread/unsupported.rs`). So three of the four primitives in this file
// have to be answered per target, the same way `clock/cpu` already is.
//
// All four refuse rather than approximate. A wall clock passed off as a
// monotonic one makes every duration computed from it quietly wrong, which is
// worse than an error a program can catch — and `time/sleep` cannot be
// approximated at all, since a browser cannot block.
//
// An embedder *could* supply real time here (`performance.now()` and
// `Date.now()` both exist in the host), but only by having elle import a
// function from JS. That would put a required import in elle's wasm ABI and
// oblige every embedder to provide it, so it is deliberately not done: the
// demo does not need timing, and a later embedder-registered primitive can
// add it without changing this file.

#[cfg(not(target_arch = "wasm32"))]
static PROCESS_EPOCH: OnceLock<Instant> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn process_epoch() -> &'static Instant {
    PROCESS_EPOCH.get_or_init(Instant::now)
}

/// Returns seconds elapsed since process start (monotonic clock)
/// (clock/monotonic)
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn prim_clock_monotonic(
    _ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    _args: &[Value],
) -> (SignalBits, Value) {
    (
        SIG_OK,
        Value::float(process_epoch().elapsed().as_secs_f64()),
    )
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn prim_clock_monotonic(
    ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    _args: &[Value],
) -> (SignalBits, Value) {
    (
        SIG_ERROR,
        ctx.error(
            "unsupported",
            "clock/monotonic: no clock on wasm32".to_string(),
        ),
    )
}

/// Returns thread CPU time in seconds
/// (clock/cpu)
#[cfg(target_arch = "wasm32")]
pub(crate) fn prim_clock_cpu(
    ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    _args: &[Value],
) -> (SignalBits, Value) {
    // No per-thread CPU clock here. Substituting wall time would make any
    // profile built on this silently wrong, so refuse instead.
    (
        SIG_ERROR,
        ctx.error(
            "unsupported",
            "clock/cpu: no thread CPU clock on wasm32".to_string(),
        ),
    )
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn prim_clock_cpu(
    ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    _args: &[Value],
) -> (SignalBits, Value) {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: clock_gettime with CLOCK_THREAD_CPUTIME_ID is always valid
    // and ts is a properly initialized timespec.
    let ret = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut ts) };
    if ret != 0 {
        return (
            SIG_ERROR,
            ctx.error("io-error", "clock/cpu: clock_gettime failed".to_string()),
        );
    }
    let secs = ts.tv_sec as f64 + ts.tv_nsec as f64 / 1_000_000_000.0;
    (SIG_OK, Value::float(secs))
}

/// Returns seconds since Unix epoch (wall clock)
/// (clock/realtime)
#[cfg(target_arch = "wasm32")]
pub(crate) fn prim_clock_realtime(
    ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    _args: &[Value],
) -> (SignalBits, Value) {
    (
        SIG_ERROR,
        ctx.error(
            "unsupported",
            "clock/realtime: no system clock on wasm32".to_string(),
        ),
    )
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn prim_clock_realtime(
    ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    _args: &[Value],
) -> (SignalBits, Value) {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => (SIG_OK, Value::float(duration.as_secs_f64())),
        Err(_) => (
            SIG_ERROR,
            ctx.error(
                "io-error",
                "clock/realtime: system clock is before Unix epoch".to_string(),
            ),
        ),
    }
}

/// Sleeps for the specified number of seconds
/// (time/sleep seconds)
///
/// Refuses on wasm32 without inspecting the argument: a blocking sleep is not
/// something this target can do for any duration, so validating first would only
/// change which error a caller gets for a call that cannot succeed either way.
#[cfg(target_arch = "wasm32")]
pub(crate) fn prim_sleep(
    ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    _args: &[Value],
) -> (SignalBits, Value) {
    (
        SIG_ERROR,
        ctx.error(
            "unsupported",
            "time/sleep: cannot block on wasm32".to_string(),
        ),
    )
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn prim_sleep(
    ctx: &mut crate::primitives::ctx::NativeCtx<'_>,
    args: &[Value],
) -> (SignalBits, Value) {
    if let Some(n) = args[0].as_int() {
        if n < 0 {
            return (
                SIG_ERROR,
                ctx.error(
                    "argument-error",
                    "time/sleep: duration must be non-negative".to_string(),
                ),
            );
        }
        std::thread::sleep(std::time::Duration::from_secs(n as u64));
        (SIG_OK, Value::NIL)
    } else if let Some(f) = args[0].as_float() {
        if f < 0.0 || !f.is_finite() {
            return (
                SIG_ERROR,
                ctx.error(
                    "argument-error",
                    "time/sleep: duration must be a finite non-negative number".to_string(),
                ),
            );
        }
        std::thread::sleep(std::time::Duration::from_secs_f64(f));
        (SIG_OK, Value::NIL)
    } else {
        (
            SIG_ERROR,
            ctx.error(
                "type-error",
                "time/sleep: argument must be a number".to_string(),
            ),
        )
    }
}

// Declarative primitive definitions for time operations
primitive! {
    "clock/monotonic" => prim_clock_monotonic {
        signal: Signal::errors(),
        doc: "Return seconds elapsed since process start (monotonic clock)",
        category: "clock",
        example: "(clock/monotonic)",
        effect: RegionEffect::Immediate,
    }
    "clock/realtime" => prim_clock_realtime {
        signal: Signal::errors(),
        doc: "Return seconds since Unix epoch (wall clock)",
        category: "clock",
        example: "(clock/realtime)",
        effect: RegionEffect::Immediate,
    }
    "clock/cpu" => prim_clock_cpu {
        signal: Signal::errors(),
        doc: "Return thread CPU time in seconds",
        category: "clock",
        example: "(clock/cpu)",
        effect: RegionEffect::Immediate,
    }
    "time/sleep" => prim_sleep {
        signal: Signal::errors(),
        arity: Arity::Exact(1),
        doc: "Sleep for the specified number of seconds (blocks the thread)",
        params: &["seconds"],
        category: "time",
        example: "(time/sleep 1.5)",
        effect: RegionEffect::Immediate,
    }
}
