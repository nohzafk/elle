#!/bin/bash
# Regenerate src/primitives/stub_wasm.rs — the wasm32 stand-ins for the
# primitives whose implementing modules are compiled out on that target.
#
# Only the *names* matter: stdlib.lisp defines wrappers (ev/spawn,
# subprocess/system, chan/select ...) whose bodies reference these
# primitives, and an unbound name is a compile error in Elle. Every name
# stays bound, to a primitive that reports `:unsupported`.
#
# Run from anywhere; paths are resolved against the repo root.
set -euo pipefail
cd "$(dirname "$0")/.."

MODULES="chan concurrency io net unix ports posix stream subprocess watch"
OUT=src/primitives/stub_wasm.rs

# Names that wasm32 implements for real, in src/primitives/stdio_wasm.rs, and so
# must NOT get a stand-in here — a duplicate registration would be two entries
# for one name. They live in a cut-out module (`ports`) but need nothing from the
# OS: they only mint a `Port` value with no descriptor.
#
# This is not a convenience. stdlib.lisp calls all three at load time
# (`(def *stdout* (parameter (port/stdout)))` and friends at top level), so a
# stand-in answering `:unsupported` takes down `init_stdlib` itself.
PROVIDED="port/stdin port/stdout port/stderr"

# Names live inside `primitive!` blocks only. A bare grep for '"x" =>' also
# hits ordinary match arms (io.rs has `"read" => libc::POLLIN`), so the
# extraction is scoped to the macro block. Aliases are collected too and
# flattened into ordinary entries: the registered symbol set is what
# matters, not which spelling is canonical.
extract() {
  for m in $MODULES; do
    for f in "src/primitives/$m.rs" src/primitives/"$m"/*.rs; do
      [ -f "$f" ] || continue
      awk '
        /primitive!/ { inb = 1 }
        inb && /^[[:space:]]*"[^"]+"[[:space:]]*=>/ {
          s = $0
          sub(/^[[:space:]]*"/, "", s)
          sub(/".*/, "", s)
          print s
        }
        inb && /aliases:/ {
          s = $0
          while (match(s, /"[^"]+"/)) {
            print substr(s, RSTART + 1, RLENGTH - 2)
            s = substr(s, RSTART + RLENGTH)
          }
        }
        inb && /^\}/ { inb = 0 }
        inb && /^\);/ { inb = 0 }
      ' "$f"
    done
  done | sort -u
}

all=$(extract)

# A PROVIDED name that is no longer extracted means the native spelling moved and
# this list went stale. Left silent, the new spelling would get a stand-in and
# stdlib.lisp would again fail to load — the exact bug PROVIDED exists to prevent.
for p in $PROVIDED; do
  if ! printf '%s\n' "$all" | grep -qxF "$p"; then
    echo "gen-wasm-stubs: PROVIDED name '$p' is not defined in any of: $MODULES" >&2
    echo "  it was probably renamed — update PROVIDED and src/primitives/stdio_wasm.rs" >&2
    exit 1
  fi
done

names=$(printf '%s\n' "$all" | grep -vxF "$(printf '%s\n' $PROVIDED)" || true)
count=$(printf '%s\n' "$names" | grep -c . || true)

# A silent zero here would emit an empty table and the failure would only
# surface much later, as an unbound name in stdlib.lisp.
if [ "$count" -lt 40 ]; then
  echo "gen-wasm-stubs: extracted only $count names — extraction is broken" >&2
  exit 1
fi

{
  cat <<'HEADER'
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
HEADER
  printf '%s\n' "$names" | while IFS= read -r n; do
    [ -n "$n" ] || continue
    printf '    "%s" => prim_unsupported { signal: Signal::errors(), arity: Arity::AtLeast(0), category: "unsupported" }\n' "$n"
  done
  echo '}'
} >"$OUT"

echo "gen-wasm-stubs: wrote $OUT with $count names"
