use std::fs;
use std::path::{ Path, PathBuf };
use std::process::{ Command, Output, Stdio };
use std::sync::atomic::{ AtomicUsize, Ordering };

static NEXT_TEMP_FILE: AtomicUsize = AtomicUsize::new(0);

fn temp_source(extension: &str, source: &str) -> PathBuf {
    let id = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
    let path = std::env
        ::temp_dir()
        .join(format!("bfx-test-{}-{}.{}", std::process::id(), id, extension));
    fs::write(&path, source).expect("write test source");
    path
}

fn run(path: &Path, input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bfx"))
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start bfx");

    {
        let stdin = child.stdin.as_mut().expect("open child stdin");
        use std::io::Write;
        stdin.write_all(input).expect("write child stdin");
    }

    child.wait_with_output().expect("wait for bfx")
}

fn run_args(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bfx")).args(args).output().expect("start bfx")
}

#[test]
fn executes_commands_loops_and_comments() {
    let path = temp_source("bf", ">+<[->+<]>.");
    let output = run(&path, &[]);
    fs::remove_file(path).expect("remove test source");

    assert!(output.status.success());
    assert_eq!(output.stdout, [1]);
    assert!(output.stderr.is_empty());
}

#[test]
fn wraps_pointer_and_cell_values() {
    let path = temp_source("bfx", "<-.+.");
    let output = run(&path, &[]);
    fs::remove_file(path).expect("remove test source");

    assert!(output.status.success());
    assert_eq!(output.stdout, [255, 0]);
}

#[test]
fn reads_input_and_uses_zero_at_eof() {
    let path = temp_source("b", ",[.,]");
    let output = run(&path, b"ab");
    fs::remove_file(path).expect("remove test source");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"ab");
}

#[test]
fn accepts_all_supported_extensions() {
    for extension in ["b", "bf", "bfx"] {
        let path = temp_source(extension, "+.");
        let output = run(&path, &[]);
        fs::remove_file(path).expect("remove test source");

        assert!(output.status.success(), "extension: {extension}");
        assert_eq!(output.stdout, [1], "extension: {extension}");
    }
}

#[test]
fn rejects_missing_argument() {
    let output = run_args(&[]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
}

#[test]
fn rejects_missing_file() {
    let path = std::env::temp_dir().join("bfx-file-that-does-not-exist.bf");
    let output = run_args(&[path.to_str().expect("temporary path is valid UTF-8")]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("does not exist"));
}

#[test]
fn rejects_directories_and_unsupported_extensions() {
    let directory = std::env::temp_dir().join(format!("bfx-test-dir-{}", std::process::id()));
    fs::create_dir_all(&directory).expect("create test directory");
    let directory_output = run_args(&[directory.to_str().expect("temporary path is valid UTF-8")]);
    fs::remove_dir(&directory).expect("remove test directory");

    assert!(!directory_output.status.success());
    assert!(
        String::from_utf8_lossy(&directory_output.stderr).contains(
            "does not exist or is not a file"
        )
    );

    let no_extension = std::env
        ::temp_dir()
        .join(format!("bfx-test-no-extension-{}", std::process::id()));
    fs::write(&no_extension, "+.").expect("write extensionless test source");
    let no_extension_output = run_args(
        &[no_extension.to_str().expect("temporary path is valid UTF-8")]
    );
    fs::remove_file(no_extension).expect("remove extensionless test source");

    assert!(!no_extension_output.status.success());
    assert!(
        String::from_utf8_lossy(&no_extension_output.stderr).contains(
            "must end with .b, .bf, or .bfx"
        )
    );

    let path = temp_source("txt", "+.");
    let output = run_args(&[path.to_str().expect("temporary path is valid UTF-8")]);
    fs::remove_file(path).expect("remove test source");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("must end with .b, .bf, or .bfx"));
}

#[test]
fn rejects_unmatched_closing_bracket() {
    let path = temp_source("bf", "]");
    let output = run(&path, &[]);
    fs::remove_file(path).expect("remove test source");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unmatched ']' at position 0"));
}
