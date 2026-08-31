//! Where wasm32's `port/write` puts its bytes.
//!
//! On a real target `println` reaches a file descriptor. wasm32 has none, so the
//! text is buffered here and the host drains it — `take` after each evaluation,
//! rather than a callback into JS per write.
//!
//! Buffering rather than calling out is deliberate. A JS callback would put a
//! required import in the module's ABI, binding every embedder to supply it, for
//! the same reason the wasm clocks refuse instead of importing `Date.now`. It
//! also keeps evaluation from re-entering JS at arbitrary points. The cost is
//! that output appears when the call returns instead of during it, which no
//! caller of a synchronous `eval` could observe anyway.
//!
//! Two streams, not one: `PortKind` already tells `Stdout` from `Stderr`, and
//! merging them would throw away a distinction the native target keeps and a UI
//! wants back.
//!
//! Not gated to wasm32 — nothing here is target-specific, and a plain unit test
//! is worth more than a mystery only a browser can reproduce.

use std::cell::RefCell;

/// Which stream text was written to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    Out,
    Err,
}

thread_local! {
    static OUT: RefCell<String> = const { RefCell::new(String::new()) };
    static ERR: RefCell<String> = const { RefCell::new(String::new()) };
}

fn with<R>(stream: Stream, f: impl FnOnce(&mut String) -> R) -> R {
    match stream {
        Stream::Out => OUT.with(|b| f(&mut b.borrow_mut())),
        Stream::Err => ERR.with(|b| f(&mut b.borrow_mut())),
    }
}

/// Append `text` to `stream`.
pub fn push(stream: Stream, text: &str) {
    with(stream, |b| b.push_str(text));
}

/// Take everything buffered on `stream`, leaving it empty.
pub fn take(stream: Stream) -> String {
    with(stream, std::mem::take)
}

/// Bytes buffered on `stream` without draining it.
pub fn len(stream: Stream) -> usize {
    with(stream, |b| b.len())
}

/// Drop both streams' contents.
pub fn clear() {
    with(Stream::Out, |b| b.clear());
    with(Stream::Err, |b| b.clear());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_drains_and_leaves_the_buffer_empty() {
        clear();
        push(Stream::Out, "a");
        push(Stream::Out, "b");
        assert_eq!(len(Stream::Out), 2);
        assert_eq!(take(Stream::Out), "ab");
        assert_eq!(take(Stream::Out), "");
        assert_eq!(len(Stream::Out), 0);
    }

    #[test]
    fn the_two_streams_do_not_mix() {
        clear();
        push(Stream::Out, "out");
        push(Stream::Err, "err");
        assert_eq!(take(Stream::Out), "out");
        assert_eq!(take(Stream::Err), "err");
    }
}
