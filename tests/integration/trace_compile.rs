// `--trace=compile` phase-timing marks.
//
// The compile pipeline and the stdlib load both emit per-phase timings. They
// belong on the project's one diagnostic switch: a second, invisible one is a
// channel nothing on the command line names, and it cannot be turned on for a
// single run of a command whose behaviour it changes.

use std::process::Command;

fn elle() -> &'static str {
    env!("CARGO_BIN_EXE_elle")
}

fn run(args: &[&str], source: &str) -> (String, std::process::ExitStatus) {
    let output = Command::new(elle())
        .args(args)
        .arg("-e")
        .arg(source)
        .output()
        .expect("spawn elle");
    (
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status,
    )
}

#[test]
fn compile_trace_marks_the_stdlib_load() {
    let (err, status) = run(&["--trace=compile"], "(+ 1 2)");
    assert!(status.success(), "elle --trace=compile exited non-zero:\n{err}");
    assert!(
        err.contains("[trace:compile] stdlib"),
        "the stdlib load must report its phases on the shared trace switch:\n{err}"
    );
}

#[test]
fn compile_trace_marks_the_user_compile() {
    let (err, status) = run(&["--trace=compile"], "(defn sq [x] (* x x)) (sq 3)");
    assert!(status.success(), "elle --trace=compile exited non-zero:\n{err}");
    assert!(
        err.contains("[trace:compile] frontend"),
        "the frontend phases must report on the same switch:\n{err}"
    );
}

#[test]
fn phase_marks_stay_quiet_without_the_keyword() {
    // A diagnostic that prints when nobody asked is a diagnostic that gets
    // filtered out and then ignored.
    let (err, status) = run(&[], "(+ 1 2)");
    assert!(status.success(), "elle exited non-zero:\n{err}");
    assert!(
        !err.contains("[trace:compile]"),
        "no phase mark may appear without --trace=compile:\n{err}"
    );
}
