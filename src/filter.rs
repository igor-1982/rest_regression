use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

#[derive(Deserialize, Debug)]
pub struct TestEntry {
    pub num_mpi: Option<usize>,
    pub num_threads: Option<usize>,
    pub tags: Option<Vec<String>>,
    pub timeout: Option<u64>,
}

pub type TestConfig = HashMap<String, TestEntry>;

pub fn load_test_config() -> TestConfig {
    let config_path = Path::new("tests.toml");
    if config_path.is_file() {
        let content = fs::read_to_string(config_path).unwrap();
        toml::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

pub fn build_all_names(input_folder: &str, test_config: &TestConfig) -> HashSet<String> {
    let mut names = HashSet::new();
    for entry in WalkDir::new(input_folder).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_dir()) {
        if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
            names.insert(name.to_lowercase());
        }
    }
    for entry in test_config.values() {
        if let Some(ref tags) = entry.tags {
            for t in tags {
                names.insert(t.to_lowercase());
            }
        }
    }
    names
}

pub fn entry_matches(entry: &DirEntry, input_folder: &str, set: &HashSet<String>, test_config: &TestConfig) -> bool {
    let relative = entry.path().strip_prefix(input_folder).unwrap_or(entry.path());
    let path_matched = relative.iter().any(|c| set.contains(&c.to_str().unwrap_or("").to_lowercase()));
    let test_name = entry.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
    let tag_matched = test_config.get(test_name)
        .and_then(|e| e.tags.as_ref())
        .map(|tags| tags.iter().any(|t| set.contains(&t.to_lowercase())))
        .unwrap_or(false);
    path_matched || tag_matched
}

pub fn remove_matched(entry: &DirEntry, input_folder: &str, set: &HashSet<String>, test_config: &TestConfig, unmatched: &mut HashSet<String>) {
    let relative = entry.path().strip_prefix(input_folder).unwrap_or(entry.path());
    for c in relative.iter() {
        let name = c.to_str().unwrap_or("").to_lowercase();
        if set.contains(&name) {
            unmatched.remove(&name);
        }
    }
    let test_name = entry.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
    if let Some(tags) = test_config.get(test_name).and_then(|e| e.tags.as_ref()) {
        for t in tags {
            let lower = t.to_lowercase();
            if set.contains(&lower) {
                unmatched.remove(&lower);
            }
        }
    }
}
