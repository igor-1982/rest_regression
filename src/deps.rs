//! Probing the availability of Python dependencies declared in tests.toml
//!
//! REST's MD/QM-MM code imports `ase` / `openmm` inside its embedded Python
//! runtime at execution time; a missing package makes the whole test fail.
//! Tests declare their needs via `requires = [...]` in tests.toml, and this
//! module checks importability (subprocess `python -c "import ..."`) so the
//! regression runner can skip those tests instead of failing them.

use std::collections::HashMap;
use std::env;
use std::process::{Command, Stdio};
use std::sync::Mutex;

static CACHE: Mutex<Option<HashMap<String, bool>>> = Mutex::new(None);

/// The Python snippet that must run cleanly for a dependency to be available.
/// `ase` pulls in numpy; `openmm` additionally needs `openmm.app` (as used by
/// REST's QM/MM code).
fn import_snippet(dep: &str) -> String {
    match dep {
        "ase" => "import numpy, ase".to_string(),
        "openmm" => "import openmm, openmm.app".to_string(),
        other => format!("import {}", other),
    }
}

/// Candidate interpreters: $REST_REGRESSION_PYTHON takes precedence, then
/// python3/python from PATH. The first interpreter that can be launched
/// decides the result (no silent fallback between existing interpreters).
fn interpreter_candidates() -> Vec<String> {
    if let Ok(py) = env::var("REST_REGRESSION_PYTHON") {
        vec![py]
    } else {
        vec!["python3".to_string(), "python".to_string()]
    }
}

fn probe(dep: &str) -> bool {
    let snippet = import_snippet(dep);
    for py in interpreter_candidates() {
        match Command::new(&py)
            .arg("-c")
            .arg(&snippet)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(status) => return status.success(),
            Err(_) => continue, // interpreter not found, try the next candidate
        }
    }
    println!("WARNING : no Python interpreter found (tried $REST_REGRESSION_PYTHON, python3, python); \
              treating dependency '{}' as missing", dep);
    false
}

/// Whether a dependency declared in tests.toml `requires` is importable.
/// The probe runs at most once per dependency (cached).
pub fn dependency_available(dep: &str) -> bool {
    let mut guard = CACHE.lock().unwrap();
    let cache = guard.get_or_insert_with(HashMap::new);
    if let Some(&v) = cache.get(dep) {
        return v;
    }
    let v = probe(dep);
    cache.insert(dep.to_string(), v);
    v
}

/// The subset of `requires` that cannot be imported in the current environment.
pub fn missing_dependencies(requires: &[String]) -> Vec<String> {
    requires.iter().filter(|d| !dependency_available(d)).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_snippets() {
        assert_eq!(import_snippet("ase"), "import numpy, ase");
        assert_eq!(import_snippet("openmm"), "import openmm, openmm.app");
        assert_eq!(import_snippet("numpy"), "import numpy");
    }

    #[test]
    fn test_absent_package_reports_unavailable() {
        // works whether or not a Python interpreter exists: a failed import
        // and a missing interpreter both count as "unavailable"
        assert!(!dependency_available("definitely_not_a_python_package_9x7"));
    }
}
