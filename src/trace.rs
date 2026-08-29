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

/// Print one phase-timing mark: `[trace:SUBSYSTEM] LABEL 12.3ms`.
pub(crate) fn phase(enabled: bool, subsystem: &str, label: &str, start: std::time::Instant) {
    if enabled {
        eprintln!("[trace:{subsystem}] {label} {:?}", start.elapsed());
    }
}
