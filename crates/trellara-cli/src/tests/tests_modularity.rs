use std::ffi::OsStr;

use super::*;

const MAX_PRODUCTION_RUST_FILE_LINES: usize = 186;
const MAX_TEST_RUST_FILE_LINES: usize = 450;

#[test]
fn production_rust_files_stay_small_enough_to_review() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root");
    let crates_dir = workspace_root.join("crates");
    let mut oversized = Vec::new();

    collect_oversized_rust_files(&crates_dir, &mut oversized);

    assert!(
        oversized.is_empty(),
        "production Rust files must stay at or below {MAX_PRODUCTION_RUST_FILE_LINES} lines; split these modules:\n{}",
        oversized.join("\n")
    );
}

#[test]
fn test_rust_files_stay_small_enough_to_review() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root");
    let crates_dir = workspace_root.join("crates");
    let mut oversized = Vec::new();

    collect_oversized_test_files(&crates_dir, &mut oversized);

    assert!(
        oversized.is_empty(),
        "test Rust files must stay at or below {MAX_TEST_RUST_FILE_LINES} lines; split these scenario modules:\n{}",
        oversized.join("\n")
    );
}

fn collect_oversized_rust_files(path: &Path, oversized: &mut Vec<String>) {
    if is_test_path(path) {
        return;
    }

    if path.is_dir() {
        for entry in fs::read_dir(path).expect("read source directory") {
            let entry = entry.expect("read source entry");
            collect_oversized_rust_files(&entry.path(), oversized);
        }
        return;
    }

    if path.extension() != Some(OsStr::new("rs")) || !is_production_source_file(path) {
        return;
    }

    let contents = fs::read_to_string(path).expect("read Rust source file");
    let line_count = contents.lines().count();
    if line_count > MAX_PRODUCTION_RUST_FILE_LINES {
        oversized.push(format!(
            "{}: {line_count} lines",
            path.strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")))
                .unwrap_or(path)
                .display()
        ));
    }
}

fn collect_oversized_test_files(path: &Path, oversized: &mut Vec<String>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("read source directory") {
            let entry = entry.expect("read source entry");
            collect_oversized_test_files(&entry.path(), oversized);
        }
        return;
    }

    if path.extension() != Some(OsStr::new("rs"))
        || !is_production_source_file(path)
        || !is_test_path(path)
    {
        return;
    }

    let contents = fs::read_to_string(path).expect("read Rust test source file");
    let line_count = contents.lines().count();
    if line_count > MAX_TEST_RUST_FILE_LINES {
        oversized.push(format!(
            "{}: {line_count} lines",
            path.strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")))
                .unwrap_or(path)
                .display()
        ));
    }
}

fn is_production_source_file(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == OsStr::new("src"))
}

fn is_test_path(path: &Path) -> bool {
    path.file_name() == Some(OsStr::new("tests.rs"))
        || path
            .components()
            .any(|component| component.as_os_str() == OsStr::new("tests"))
}
