//! Source mounted in memory, consulted before the filesystem.
//!
//! `include` and `import-file` both resolve a spec to a path and then read that
//! path. On wasm32 there is no path to read: `std::fs` compiles but every call
//! fails, so a multi-file Elle program cannot be loaded at all. Mounting the
//! sources gives the resolver something to find without any per-target code in
//! the compiler or the module primitive.
//!
//! Mounted source *shadows* a real file at the same path. That is deliberate,
//! and the reason this is not gated to wasm32: it lets the same mechanism be
//! tested natively, in-process, against the real resolver — and a test that has
//! to be run through a browser to say anything is a test that stops being run.
//!
//! Thread-local, following the other per-thread stores in this crate. Not
//! per-`Runtime`: the two read sites are free functions reached from deep in the
//! compiler with no instance in scope, and threading one through every caller
//! would cost more than the isolation is worth here. One thread holds one
//! program's sources, which is what both the wasm module and a native test want.

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static MOUNTED: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Mount `contents` at `path`, shadowing any real file there.
///
/// `path` is matched against the spec as written in the source, so mount
/// `"src/loader.lisp"` for `(include "src/loader.lisp")`.
pub fn mount(path: impl Into<String>, contents: impl Into<String>) {
    MOUNTED.with(|m| m.borrow_mut().insert(path.into(), contents.into()));
}

/// Drop everything mounted on this thread.
pub fn unmount_all() {
    MOUNTED.with(|m| m.borrow_mut().clear());
}

/// Whether `path` names mounted source. Used by the resolver, which must answer
/// this without reading.
pub fn is_mounted(path: &str) -> bool {
    MOUNTED.with(|m| m.borrow().contains_key(path))
}

/// The source mounted at `path`, if any.
pub fn read(path: &str) -> Option<String> {
    MOUNTED.with(|m| m.borrow().get(path).cloned())
}

/// How many sources are mounted on this thread.
pub fn mounted_count() -> usize {
    MOUNTED.with(|m| m.borrow().len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::eval_file;
    use crate::runtime::Runtime;

    /// Evaluate `source` against a fresh runtime, with nothing left mounted after.
    fn eval_isolated(source: &str) -> Result<String, String> {
        let mut rt = Runtime::new();
        let (vm, symbols, cctx) = rt.parts();
        eval_file(source, symbols, vm, cctx, "<vfs-test>")
            .map(|v| format!("{}", v.display_with(Some(symbols))))
            .map_err(|e| e.to_string())
    }

    #[test]
    fn include_reads_mounted_source() {
        unmount_all();
        mount("mounted/lib.lisp", "(def answer 42)");
        let got = eval_isolated("(include \"mounted/lib.lisp\") answer");
        unmount_all();
        assert_eq!(got.as_deref(), Ok("42"));
    }

    #[test]
    fn import_file_reads_mounted_source() {
        unmount_all();
        mount("mounted/mod.lisp", "(def double (fn [x] (* x 2))) [double]");
        let got = eval_isolated("(def M (import-file \"mounted/mod.lisp\")) ((get M 0) 21)");
        unmount_all();
        assert_eq!(got.as_deref(), Ok("42"));
    }

    /// A mounted source may pull in another one — the case the loader needs,
    /// where an included file includes further files.
    #[test]
    fn mounted_source_can_include_mounted_source() {
        unmount_all();
        mount("a.lisp", "(include \"b.lisp\") (def a (+ b 1))");
        mount("b.lisp", "(def b 41)");
        let got = eval_isolated("(include \"a.lisp\") a");
        unmount_all();
        assert_eq!(got.as_deref(), Ok("42"));
    }

    #[test]
    fn unmounted_path_still_fails() {
        unmount_all();
        let got = eval_isolated("(include \"nowhere/absent.lisp\") 1");
        assert!(
            got.as_ref().is_err_and(|e| e.contains("not found")),
            "expected a not-found error, got {got:?}"
        );
    }
}
