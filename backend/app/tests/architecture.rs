//! Architecture guards of the domain-first layout.
//!
//! - Bounded contexts (Identity, Publishing, Discussion) depend on the shared
//!   kernel only, never on each other nor on the composition root (Cargo).
//! - Inside a context, `domain` is framework-free and knows no outer layer,
//!   and `application` knows neither SQL nor HTTP (source scan).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

/// Package names of the bounded contexts, with their source directory.
const CONTEXTS: [(&str, &str); 3] = [
    ("xetaravel-identity", "identity"),
    ("xetaravel-publishing", "publishing"),
    ("xetaravel-discussion", "discussion"),
];

/// Internal packages a context must never depend on (besides the other contexts).
const OUTER_PACKAGES: [&str; 2] = ["xetaravel-app", "migration"];

/// Paths a domain layer must never mention.
const DOMAIN_FORBIDDEN: [&str; 7] = [
    "sea_orm",
    "axum",
    "serde",
    "ts_rs",
    "crate::application",
    "crate::infrastructure",
    "crate::http",
];

/// Paths an application layer must never mention.
const APPLICATION_FORBIDDEN: [&str; 4] =
    ["sea_orm", "axum", "crate::infrastructure", "crate::http"];

/// Returns the `backend/` directory.
fn backend_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("app crate lives in backend/")
        .to_path_buf()
}

/// Runs `cargo metadata` on the workspace (without external packages).
fn metadata() -> Value {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(backend_dir())
        .output()
        .expect("cannot run cargo metadata");
    assert!(output.status.success(), "cargo metadata failed");
    serde_json::from_slice(&output.stdout).expect("invalid cargo metadata")
}

/// Returns the names of the normal (and build) dependencies of `package`.
fn runtime_dependencies(metadata: &Value, package: &str) -> Vec<String> {
    let package = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == package)
        .unwrap_or_else(|| panic!("package {package} not found"));
    package["dependencies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["kind"].as_str() != Some("dev"))
        .map(|d| d["name"].as_str().unwrap().to_owned())
        .collect()
}

/// Lists every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap_or_else(|_| panic!("missing {}", dir.display())) {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(rust_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files
}

/// Returns `file:line: text` for each code line of `dir` mentioning a forbidden path.
fn violations(dir: &Path, forbidden: &[&str]) -> Vec<String> {
    let files = rust_files(dir);
    assert!(!files.is_empty(), "no sources in {}", dir.display());

    files
        .iter()
        .flat_map(|file| {
            let source = fs::read_to_string(file).unwrap();
            source
                .lines()
                .enumerate()
                .filter(|(_, line)| !line.trim_start().starts_with("//"))
                .filter(|(_, line)| forbidden.iter().any(|path| line.contains(path)))
                .map(|(n, line)| format!("{}:{}: {}", file.display(), n + 1, line.trim()))
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn contexts_only_depend_on_the_kernel() {
    let metadata = metadata();
    for (package, _) in CONTEXTS {
        let dependencies = runtime_dependencies(&metadata, package);
        assert!(
            dependencies.iter().any(|d| d == "xetaravel-kernel"),
            "{package} should use the shared kernel"
        );
        for (other, _) in CONTEXTS.iter().filter(|(other, _)| *other != package) {
            assert!(
                !dependencies.iter().any(|d| d == other),
                "{package} must not depend on {other}"
            );
        }
        for outer in OUTER_PACKAGES {
            assert!(
                !dependencies.iter().any(|d| d == outer),
                "{package} must not depend on {outer}"
            );
        }
    }
}

#[test]
fn kernel_depends_on_no_context() {
    let dependencies = runtime_dependencies(&metadata(), "xetaravel-kernel");
    let internal: Vec<_> = dependencies
        .iter()
        .filter(|d| d.starts_with("xetaravel-") || *d == "migration")
        .collect();
    assert!(internal.is_empty(), "kernel depends on {internal:?}");
}

#[test]
fn domain_layers_are_framework_free() {
    for (_, dir) in CONTEXTS {
        let found = violations(
            &backend_dir().join(dir).join("src/domain"),
            &DOMAIN_FORBIDDEN,
        );
        assert!(found.is_empty(), "forbidden imports:\n{}", found.join("\n"));
    }
}

#[test]
fn application_layers_know_neither_sql_nor_http() {
    for (_, dir) in CONTEXTS {
        let found = violations(
            &backend_dir().join(dir).join("src/application"),
            &APPLICATION_FORBIDDEN,
        );
        assert!(found.is_empty(), "forbidden imports:\n{}", found.join("\n"));
    }
}
