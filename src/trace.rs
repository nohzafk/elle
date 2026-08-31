//! Trace macro for runtime-gated debug output.
//!
//! Gates on the VM's `runtime_config.has_trace_bit` — one relaxed atomic load
//! of this instance's shared trace cell, no HashSet lookup.
//!
//! Format: `[trace:SUBSYSTEM] message` for easy grep filtering.

/// Emit a trace message to stderr if the given trace bit is active.
///
/// The first argument is a reference to the VM (or anything with a
/// `runtime_config` field). The second is a trace bit constant from
/// `crate::config::trace_bits`. Remaining arguments are passed to
/// `eprintln!`.
///
/// Hot-path cost when tracing is off: one bitwise AND + branch.
#[macro_export]
macro_rules! etrace {
    ($vm:expr, $bit:expr, $subsystem:expr, $($arg:tt)*) => {
        if $vm.runtime_config.has_trace_bit($bit) {
            eprintln!(concat!("[trace:", $subsystem, "] {}"), format_args!($($arg)*));
        }
    };
}

/// True when `--trace=compile` is active.
///
/// Compile phases gate on the static CLI config, not on a VM's trace cell:
/// the file frontend takes only `symbols`/`cctx`, so a phase mark fires where
/// no `&VM` is in scope. This is the gating the `[trace:regions]` dumps in
/// `compile_file_inner` already use.
pub(crate) fn compile() -> bool {
    crate::config::get().trace_bits() & crate::config::trace_bits::COMPILE != 0
}

/// The start of a phase, as [`stamp`] recorded it.
///
/// wasm32-unknown-unknown has no clock. `Instant::now()` still *compiles* there
/// — it is the same `std` API on every target — and panics with "time not
/// implemented on this platform" the moment it runs. Since compile-phase marks
/// sit on the ordinary `compile_file` path, that panic is reached by any `eval`
/// at all, so the timing has to degrade to nothing rather than be attempted.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type Stamp = std::time::Instant;
#[cfg(target_arch = "wasm32")]
pub(crate) type Stamp = ();

/// Record the start of a phase. Always call this instead of `Instant::now()` for
/// anything that feeds [`phase`]; see [`Stamp`] for why.
pub(crate) fn stamp() -> Stamp {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::Instant::now()
    }
    #[cfg(target_arch = "wasm32")]
    {}
}

/// Print one phase-timing mark: `[trace:SUBSYSTEM] LABEL 12.3ms`.
///
/// On wasm32 the duration is replaced by `(no clock)`: the phase boundaries are
/// still worth seeing, and they are all the platform can report.
pub(crate) fn phase(enabled: bool, subsystem: &str, label: &str, start: Stamp) {
    if !enabled {
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("[trace:{subsystem}] {label} {:?}", start.elapsed());
    #[cfg(target_arch = "wasm32")]
    {
        let () = start;
        eprintln!("[trace:{subsystem}] {label} (no clock)");
    }
}
