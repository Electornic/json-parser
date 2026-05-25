use std::fs;
use std::path::{Path, PathBuf};

use json_parser::parse;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn fixtures_by_prefix(prefix: &str) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(fixtures_dir())
        .expect("fixtures directory readable")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(prefix))
                .unwrap_or(false)
        })
        .collect();
    out.sort();
    out
}

#[test]
fn y_fixtures_all_accepted() {
    let mut failures = Vec::new();
    for path in fixtures_by_prefix("y_") {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let input = fs::read_to_string(&path).expect("fixture is UTF-8");
        if let Err(e) = parse(&input) {
            failures.push(format!("{name}: expected accept, got error: {e}"));
        }
    }
    assert!(failures.is_empty(), "y_ failures:\n{}", failures.join("\n"));
}

#[test]
fn n_fixtures_all_rejected() {
    let mut failures = Vec::new();
    for path in fixtures_by_prefix("n_") {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let input = fs::read_to_string(&path).expect("fixture is UTF-8");
        if let Ok(value) = parse(&input) {
            failures.push(format!("{name}: expected reject, got: {value:?}"));
        }
    }
    assert!(failures.is_empty(), "n_ failures:\n{}", failures.join("\n"));
}

#[test]
fn fixture_count_sanity() {
    let y = fixtures_by_prefix("y_");
    let n = fixtures_by_prefix("n_");
    assert!(y.len() >= 10, "expected at least 10 y_ fixtures, found {}", y.len());
    assert!(n.len() >= 10, "expected at least 10 n_ fixtures, found {}", n.len());
}
