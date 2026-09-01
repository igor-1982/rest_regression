//! JSON-reference comparison mode, configured by an optional per-test `config.toml`.
//!
//! A test folder switches from the default log scraping (`collect_results`/
//! `compare_results` in `main.rs`) to json comparison by providing
//! ```toml
//! [config]
//! reference = "json"
//! fields = [ "scf_energy", "thermo/cv_vib" ]   // required paths, optional
//!
//! [tolerance]                                   // user overrides, optional
//! "thermo/zpe" = 1.0
//! ```
//! The REST input card must set `outname = "reference"` so that the dumped
//! json is `reference.json`; the reference log is still produced by shell
//! redirection (`rest -i ctrl.in > reference.log`).
//!
//! Comparison rules (see docs/config-toml.md for the user-facing description):
//! - only leaf fields present in BOTH reference.json and the output json are
//!   compared (the intersection), so new fields added to future REST versions
//!   never break old references;
//! - `fields` and every `[tolerance]` key are required: they must resolve in
//!   both documents or the test fails;
//! - tolerance = user `[tolerance]` > `default_tolerance` table > defaults,
//!   resolved in the spirit of numpy's isclose:
//!   |out - ref| <= atol + rtol * |ref|;
//! - strings: case-insensitive equality; bool/null: equality; arrays: same
//!   length and elementwise comparison; any type mismatch fails.

use anyhow::Context;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

/// Which reference a test compares its output against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceMode {
    Log,
    Json,
}

/// A tolerance specification: a single value sets `atol` (with `rtol` at its
/// default), a pair is `[atol, rtol]`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TolSpec {
    Atol(f64),
    AtolRtol([f64; 2]),
}

impl TolSpec {
    /// Resolve into `(atol, rtol)`; a single value keeps the default `rtol`.
    pub fn to_atol_rtol(self) -> (f64, f64) {
        match self {
            TolSpec::Atol(atol) => (atol, DEFAULT_RTOL),
            TolSpec::AtolRtol([atol, rtol]) => (atol, rtol),
        }
    }

    /// Both components must be finite and non-negative (0 is allowed).
    pub fn is_valid(&self) -> bool {
        match *self {
            TolSpec::Atol(a) => a.is_finite() && a >= 0.0,
            TolSpec::AtolRtol([a, r]) => a.is_finite() && a >= 0.0 && r.is_finite() && r >= 0.0,
        }
    }
}

/// Content of a test folder's optional `config.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TestConfigFile {
    #[serde(default)]
    pub config: Option<ConfigSection>,
    #[serde(default)]
    pub tolerance: Option<HashMap<String, TolSpec>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigSection {
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub fields: Option<Vec<String>>,
}

impl TestConfigFile {
    /// `log` unless `[config].reference = "json"`.
    pub fn reference_mode(&self) -> ReferenceMode {
        if let Some(cfg) = &self.config {
            if let Some(r) = &cfg.reference {
                if r.eq_ignore_ascii_case("json") {
                    return ReferenceMode::Json;
                }
            }
        }
        ReferenceMode::Log
    }

    /// `[config].fields` united with the `[tolerance]` keys (which are
    /// implicitly required), deduplicated and sorted.
    pub fn required_fields(&self) -> Vec<String> {
        let mut fields: Vec<String> = vec![];
        if let Some(cfg) = &self.config {
            if let Some(f) = &cfg.fields {
                fields.extend(f.iter().cloned());
            }
        }
        if let Some(tol) = &self.tolerance {
            fields.extend(tol.keys().cloned());
        }
        fields.sort();
        fields.dedup();
        fields
    }

    pub fn tolerance_overrides(&self) -> HashMap<String, TolSpec> {
        self.tolerance.clone().unwrap_or_default()
    }
}

/// Read `<test_dir>/config.toml`. `Ok(None)` if the file does not exist.
/// Parse errors, or `reference` outside {json, log}, abort the run: a
/// malformed test should be reported loudly, like a missing `ctrl.in`.
pub fn load_per_test_config(test_dir: &Path) -> anyhow::Result<Option<TestConfigFile>> {
    let path = test_dir.join("config.toml");
    if !path.is_file() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("cannot read config.toml in {}", test_dir.display()))?;
    let cfg: TestConfigFile = toml::from_str(&content).with_context(|| format!("cannot parse {}", path.display()))?;
    if let Some(cfg_sec) = &cfg.config {
        if let Some(r) = &cfg_sec.reference {
            if !r.eq_ignore_ascii_case("json") && !r.eq_ignore_ascii_case("log") {
                anyhow::bail!("{}: [config].reference must be \"json\" or \"log\", got \"{}\"", path.display(), r);
            }
        }
    }
    if let Some(tol) = &cfg.tolerance {
        for (key, spec) in tol {
            if !spec.is_valid() {
                anyhow::bail!("{}: [tolerance].{} must be a non-negative finite atol, or [atol, rtol] of such values", path.display(), key);
            }
        }
    }
    Ok(Some(cfg))
}

/// Parse a json file; the document root must be an object.
pub fn load_json_file(path: &Path) -> anyhow::Result<Value> {
    let content = fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    let value: Value = serde_json::from_str(&content).with_context(|| format!("cannot parse {}", path.display()))?;
    if !value.is_object() {
        anyhow::bail!("{}: expected a json object at the document root", path.display());
    }
    Ok(value)
}

/// Default absolute tolerance for any path not present in the default table
/// and not overridden by the user.
pub const DEFAULT_ATOL: f64 = 1.0e-8;

/// Default relative tolerance, used whenever only `atol` is specified
/// (single-value tolerance entries and the fallback).
pub const DEFAULT_RTOL: f64 = 1.0e-5;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DefaultToleranceFile {
    tolerance: HashMap<String, TolSpec>,
}

/// The default tolerance table, embedded into the binary at compile time from
/// `src/default_tolerance.toml` and parsed once on first use.
static DEFAULT_TOLERANCE: LazyLock<HashMap<String, TolSpec>> = LazyLock::new(|| {
    let file: DefaultToleranceFile = toml::from_str(include_str!("default_tolerance.toml"))
        .expect("src/default_tolerance.toml must be a [tolerance] table of path = atol or path = [atol, rtol] entries");
    assert!(file.tolerance.values().all(|spec| spec.is_valid()), "invalid tolerance entry in src/default_tolerance.toml");
    file.tolerance
});

/// Default tolerance for a json field path, carried over from the log-mode
/// tolerances in `compare_results` (main.rs) for the observables that exist
/// there (see `src/default_tolerance.toml`). Single values set `atol`;
/// anything not listed falls back to `(DEFAULT_ATOL, DEFAULT_RTOL)`.
pub fn default_tolerance(path: &str) -> TolSpec {
    DEFAULT_TOLERANCE.get(path).copied().unwrap_or(TolSpec::Atol(DEFAULT_ATOL))
}

/// Navigate a `/`-separated path through nested objects. `None` on any miss
/// or when a segment tries to traverse a non-object.
pub fn resolve_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = root;
    for segment in path.split('/') {
        current = current.get(segment)?;
    }
    Some(current)
}

/// Flatten a json object into `leaf_path -> &Value`, where a leaf is any
/// non-object value (arrays count as leaves and are compared elementwise).
/// Empty objects contribute nothing.
pub fn flatten_leaves(value: &Value) -> BTreeMap<String, &Value> {
    let mut leaves = BTreeMap::new();
    fn walk<'a>(prefix: &str, value: &'a Value, leaves: &mut BTreeMap<String, &'a Value>) {
        if let Some(map) = value.as_object() {
            for (key, sub) in map {
                let path = if prefix.is_empty() { key.clone() } else { format!("{}/{}", prefix, key) };
                walk(&path, sub, leaves);
            }
        } else if !prefix.is_empty() {
            leaves.insert(prefix.to_string(), value);
        }
    }
    walk("", value, &mut leaves);
    leaves
}

/// Type-dispatched agreement of two json values in the spirit of numpy's
/// `isclose`: numbers agree when `|a - b| <= atol + rtol * |b|`, i.e. the
/// relative term is taken with respect to the second argument, the reference.
/// Strings compare case-insensitively; arrays elementwise under the same
/// `(atol, rtol)`; any type mismatch fails.
pub fn values_agree(a: &Value, b: &Value, atol: f64, rtol: f64) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(x), Some(y)) => (x - y).abs() <= atol + rtol * y.abs(),
            _ => false,
        },
        (Value::String(x), Value::String(y)) => x.eq_ignore_ascii_case(y),
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Null, Value::Null) => true,
        (Value::Array(xs), Value::Array(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys.iter()).all(|(x, y)| values_agree(x, y, atol, rtol))
        },
        _ => false,
    }
}

/// Compare reference and output json documents, printing a per-field report
/// in the style of `compare_results`. `required` paths (unioned with the
/// `[tolerance]` override keys, which are implicitly required) must resolve
/// in both documents; the leaves present in both documents are then compared
/// with `overrides > default_tolerance > (DEFAULT_ATOL, DEFAULT_RTOL)`, using
/// the numpy-isclose predicate of `values_agree`.
pub fn compare_json(reference: &Value, output: &Value, required: &[String], overrides: &HashMap<String, TolSpec>) -> bool {
    let ref_leaves = flatten_leaves(reference);
    let out_leaves = flatten_leaves(output);
    let mut is_same = true;

    println!("                                     Current Value vs      Reference Value");

    let mut required_all: Vec<&str> = required.iter().map(|s| s.as_str()).collect();
    required_all.extend(overrides.keys().map(|s| s.as_str()));
    required_all.sort_unstable();
    required_all.dedup();
    for path in required_all {
        if resolve_path(reference, path).is_none() {
            is_same = false;
            println!("WARNING : {:30}: required field not found in reference.json", path);
        }
        if resolve_path(output, path).is_none() {
            is_same = false;
            println!("WARNING : {:30}: required field not found in output json", path);
        }
    }

    let mut n_compared: usize = 0;
    let mut n_failed: usize = 0;
    for (path, ref_value) in ref_leaves.iter() {
        let Some(out_value) = out_leaves.get(path) else { continue };
        let (atol, rtol) = match overrides.get(path) {
            Some(spec) => spec.to_atol_rtol(),
            None => default_tolerance(path).to_atol_rtol(),
        };
        n_compared += 1;
        if values_agree(out_value, ref_value, atol, rtol) {
            println!("Pass    : {:30}: {:?}  = {:?} (atol {:.1e}, rtol {:.1e})", path, out_value, ref_value, atol, rtol);
        } else {
            is_same = false;
            n_failed += 1;
            println!("WARNING : {:30}: {:?} != {:?} (atol {:.1e}, rtol {:.1e})", path, out_value, ref_value, atol, rtol);
        }
    }

    println!("JSON fields: {} compared, {} failed", n_compared, n_failed);
    is_same
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_tolerance() {
        assert_eq!(default_tolerance("scf_energy"), TolSpec::Atol(1.0e-5));
        assert_eq!(default_tolerance("dipole_debye"), TolSpec::Atol(1.0e-4));
        assert_eq!(default_tolerance("gradient"), TolSpec::Atol(1.0e-4));
        assert_eq!(default_tolerance("hoco_luco_gap_ev"), TolSpec::Atol(1.0e-5));
        assert_eq!(default_tolerance("gw/homo_qp"), TolSpec::Atol(1.0e-5));
        assert_eq!(default_tolerance("bse/first_excitation"), TolSpec::Atol(1.0e-5));
        assert_eq!(default_tolerance("tddft/energies"), TolSpec::Atol(5.0e-5));
        assert_eq!(default_tolerance("tddft/osc"), TolSpec::Atol(1.0e-4));
        assert_eq!(default_tolerance("hessian/total_max_abs"), TolSpec::Atol(1.0e-5));
        assert_eq!(default_tolerance("hessian/frequencies_cm"), TolSpec::Atol(2.0));
        assert_eq!(default_tolerance("analdrv/frequencies_cm"), TolSpec::Atol(0.1));
        // not present in the log-mode tolerance table -> fallback (atol only, default rtol)
        assert_eq!(default_tolerance("thermo/cv_vib"), TolSpec::Atol(DEFAULT_ATOL));
        assert_eq!(default_tolerance("nuc_energy"), TolSpec::Atol(DEFAULT_ATOL));
        assert_eq!(default_tolerance("anything/else"), TolSpec::Atol(DEFAULT_ATOL));
        assert_eq!(TolSpec::Atol(1.0e-4).to_atol_rtol(), (1.0e-4, DEFAULT_RTOL));
        assert_eq!(TolSpec::AtolRtol([1.0e-8, 1.0e-3]).to_atol_rtol(), (1.0e-8, 1.0e-3));
        assert!(TolSpec::Atol(0.0).is_valid());
        assert!(!TolSpec::Atol(-1.0).is_valid());
        assert!(!TolSpec::AtolRtol([1.0e-8, -1.0e-5]).is_valid());
        // asset integrity: every entry of default_tolerance.toml must parse and
        // be valid, and the table size must not shrink unnoticed (a dropped key
        // silently loosens the comparison to the fallback)
        assert_eq!(DEFAULT_TOLERANCE.len(), 15);
        for spec in DEFAULT_TOLERANCE.values() {
            assert!(spec.is_valid());
        }
    }

    #[test]
    fn test_resolve_path() {
        let doc = json!({ "thermo": { "cv_vib": 1.5 }, "scf_energy": -56.2 });
        assert_eq!(resolve_path(&doc, "thermo/cv_vib"), Some(&json!(1.5)));
        assert_eq!(resolve_path(&doc, "scf_energy"), Some(&json!(-56.2)));
        // an object itself resolves (legal as a required field anchor)
        assert_eq!(resolve_path(&doc, "thermo"), Some(&json!({ "cv_vib": 1.5 })));
        assert_eq!(resolve_path(&doc, "thermo/zpe"), None);
        assert_eq!(resolve_path(&doc, "nope"), None);
        assert_eq!(resolve_path(&doc, "thermo/cv_vib/0"), None);
    }

    #[test]
    fn test_flatten_leaves() {
        let doc = json!({
            "scf_energy": -56.2,
            "energies": { "total_energy": [-56.3] },
            "analdrv": { "frequencies_cm": [1.0, 2.0], "modes_trv": ["TR", "V"] },
            "empty": {},
        });
        let leaves = flatten_leaves(&doc);
        assert_eq!(leaves.len(), 4);
        assert_eq!(leaves["scf_energy"], &json!(-56.2));
        assert_eq!(leaves["energies/total_energy"], &json!([-56.3]));
        assert_eq!(leaves["analdrv/frequencies_cm"], &json!([1.0, 2.0]));
        assert_eq!(leaves["analdrv/modes_trv"], &json!(["TR", "V"]));
        assert!(!leaves.contains_key("empty"));
    }

    #[test]
    fn test_values_agree() {
        // pure atol checks (rtol = 0)
        assert!(values_agree(&json!(1.0), &json!(1.0 + 9.0e-7), 1.0e-6, 0.0));
        assert!(!values_agree(&json!(1.0), &json!(1.0 + 1.1e-6), 1.0e-6, 0.0));
        // isclose: |a - b| <= atol + rtol * |b|, rtol taken on the reference (second arg)
        assert!(values_agree(&json!(100.5), &json!(100.0), 0.0, 1.0e-2));
        assert!(!values_agree(&json!(101.5), &json!(100.0), 0.0, 1.0e-2));
        // same deviation 1.5: passes with |ref| = 200, fails with |ref| = 100
        assert!(values_agree(&json!(201.5), &json!(200.0), 0.0, 1.0e-2));
        assert!(!values_agree(&json!(101.5), &json!(100.0), 0.0, 1.0e-2));
        // atol and rtol add up
        assert!(values_agree(&json!(1.0 + 1.9e-3), &json!(1.0), 1.0e-3, 1.0e-3));
        assert!(!values_agree(&json!(1.0 + 2.1e-3), &json!(1.0), 1.0e-3, 1.0e-3));
        // strings are case-insensitive
        assert!(values_agree(&json!("C3v"), &json!("C3V"), 0.0, 0.0));
        assert!(!values_agree(&json!("C3v"), &json!("Cs"), 0.0, 0.0));
        assert!(values_agree(&json!(true), &json!(true), 0.0, 0.0));
        assert!(!values_agree(&json!(true), &json!(false), 0.0, 0.0));
        assert!(values_agree(&json!(null), &json!(null), 0.0, 0.0));
        // arrays: elementwise, and a length mismatch fails
        assert!(values_agree(&json!([1.0, 2.0]), &json!([1.0, 2.0 + 5.0e-7]), 1.0e-6, 0.0));
        assert!(!values_agree(&json!([1.0, 2.0]), &json!([1.0]), 1.0e-6, 0.0));
        assert!(values_agree(&json!(["TR", "V"]), &json!(["tr", "v"]), 0.0, 0.0));
        // type mismatch fails
        assert!(!values_agree(&json!(1.0), &json!("1.0"), 1.0e-6, 0.0));
        assert!(!values_agree(&json!(null), &json!(0.0), 1.0e-6, 0.0));
    }

    fn demo_reference() -> Value {
        json!({
            "scf_energy": -56.147740317827,
            "dipole_debye": [1.38, 0.21, -3.90],
            "analdrv": { "frequencies_cm": [1080.76, 3520.25] },
            "thermo": { "cv_vib": 1.46, "zpe": 92793.11, "point_group": "C3v" },
        })
    }

    #[test]
    fn test_compare_json_identical() {
        let doc = demo_reference();
        assert!(compare_json(&doc, &doc, &[], &HashMap::new()));
    }

    #[test]
    fn test_compare_json_perturbation() {
        let reference = demo_reference();
        let mut output = demo_reference();
        output["scf_energy"] = json!(-56.05); // dev 0.1 > 1e-5
        assert!(!compare_json(&reference, &output, &[], &HashMap::new()));
        let ok = compare_json(&reference, &demo_reference(), &[], &HashMap::new());
        assert!(ok);
    }

    #[test]
    fn test_compare_json_intersection() {
        // extra field only in the output must be ignored
        let reference = demo_reference();
        let mut output = demo_reference();
        output["brand_new_field"] = json!(42.0);
        assert!(compare_json(&reference, &output, &[], &HashMap::new()));
        // extra field only in the reference is likewise not compared,
        // unless it is required
        let mut reference2 = demo_reference();
        reference2["future_field"] = json!(1.0);
        assert!(compare_json(&reference2, &demo_reference(), &[], &HashMap::new()));
        assert!(!compare_json(&reference2, &demo_reference(), &["future_field".to_string()], &HashMap::new()));
    }

    #[test]
    fn test_compare_json_required_missing() {
        let reference = demo_reference();
        let mut output = demo_reference();
        output["thermo"].as_object_mut().unwrap().remove("cv_vib");
        assert!(!compare_json(&reference, &output, &["thermo/cv_vib".to_string()], &HashMap::new()));
    }

    #[test]
    fn test_compare_json_tolerance_override() {
        let reference = demo_reference();
        let mut output = demo_reference();
        output["thermo"]["cv_vib"] = json!(1.4615); // dev 1.5e-3
        // default fallback (atol 1e-8, rtol 1e-5): bound ~1.5e-5, fails
        assert!(!compare_json(&reference, &output, &[], &HashMap::new()));
        // single value: atol override
        let mut overrides = HashMap::new();
        overrides.insert("thermo/cv_vib".to_string(), TolSpec::Atol(1.0e-2));
        assert!(compare_json(&reference, &output, &[], &overrides));
        // pair [atol, rtol]: loose rtol passes, tight pure-atol does not
        let mut overrides = HashMap::new();
        overrides.insert("thermo/cv_vib".to_string(), TolSpec::AtolRtol([1.0e-8, 1.0e-2]));
        assert!(compare_json(&reference, &output, &[], &overrides));
        let mut overrides = HashMap::new();
        overrides.insert("thermo/cv_vib".to_string(), TolSpec::AtolRtol([1.0e-2, 0.0]));
        assert!(compare_json(&reference, &output, &[], &overrides));
        let mut overrides = HashMap::new();
        overrides.insert("thermo/cv_vib".to_string(), TolSpec::AtolRtol([1.0e-6, 0.0]));
        assert!(!compare_json(&reference, &output, &[], &overrides));
        // a tolerance key is automatically a required field
        let mut overrides2 = HashMap::new();
        overrides2.insert("thermo/not_there".to_string(), TolSpec::Atol(1.0e-2));
        assert!(!compare_json(&reference, &demo_reference(), &[], &overrides2));
    }

    #[test]
    fn test_load_per_test_config() -> anyhow::Result<()> {
        let tmp = std::env::temp_dir().join("rest_regression_json_ref_test");
        fs::create_dir_all(&tmp)?;

        // missing file -> None, log mode
        fs::remove_file(tmp.join("config.toml")).ok();
        let cfg = load_per_test_config(&tmp)?;
        assert!(cfg.is_none());

        // valid file: single-value tolerance (atol) and [atol, rtol] pair
        fs::write(
            tmp.join("config.toml"),
            r#"
[config]
reference = "json"
fields = ["scf_energy", "thermo/zpe"]

[tolerance]
"thermo/zpe" = 1.0
"thermo/cp_total" = [1e-3, 1e-6]
"#,
        )?;
        let cfg = load_per_test_config(&tmp)?.expect("config should parse");
        assert_eq!(cfg.reference_mode(), ReferenceMode::Json);
        assert_eq!(cfg.required_fields(), vec!["scf_energy".to_string(), "thermo/cp_total".to_string(), "thermo/zpe".to_string()]);
        assert_eq!(cfg.tolerance_overrides().get("thermo/zpe"), Some(&TolSpec::Atol(1.0)));
        assert_eq!(cfg.tolerance_overrides().get("thermo/cp_total"), Some(&TolSpec::AtolRtol([1.0e-3, 1.0e-6])));

        // invalid reference value
        fs::write(tmp.join("config.toml"), "[config]\nreference = \"jsonx\"\n")?;
        assert!(load_per_test_config(&tmp).is_err());

        // unknown key rejected
        fs::write(tmp.join("config.toml"), "[config]\nreference = \"json\"\nrefference = \"log\"\n")?;
        assert!(load_per_test_config(&tmp).is_err());

        // malformed tolerance values rejected
        fs::write(tmp.join("config.toml"), "[tolerance]\n\"scf_energy\" = [1e-5, 1e-6, 1e-7]\n")?;
        assert!(load_per_test_config(&tmp).is_err());
        fs::write(tmp.join("config.toml"), "[tolerance]\n\"scf_energy\" = -1e-5\n")?;
        assert!(load_per_test_config(&tmp).is_err());

        // no [config] section -> log mode
        fs::write(tmp.join("config.toml"), "[tolerance]\n\"scf_energy\" = 1e-4\n")?;
        let cfg = load_per_test_config(&tmp)?.expect("config should parse");
        assert_eq!(cfg.reference_mode(), ReferenceMode::Log);
        assert_eq!(cfg.required_fields(), vec!["scf_energy".to_string()]);
        assert_eq!(default_tolerance("scf_energy"), TolSpec::Atol(1.0e-5)); // table wins, not the override

        fs::remove_file(tmp.join("config.toml")).ok();
        Ok(())
    }

    #[test]
    fn test_compare_real_reference_json() -> anyhow::Result<()> {
        // integration against the real example test, following the pattern of
        // test_tddft_collect_results in main.rs
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("bench_pool/analdrv/NH3_RHF_hess");
        if !dir.join("reference.json").is_file() {
            return Ok(()); // example test not present (e.g. partial checkout)
        }
        let cfg = load_per_test_config(&dir)?.expect("example test must carry a config.toml");
        assert_eq!(cfg.reference_mode(), ReferenceMode::Json);
        let reference = load_json_file(&dir.join("reference.json"))?;

        // self-comparison passes
        assert!(compare_json(&reference, &reference, &cfg.required_fields(), &cfg.tolerance_overrides()));

        // a mutation beyond the default fallback tolerance fails
        let mut output = reference.clone();
        output["thermo"]["cv_vib"] = json!(reference["thermo"]["cv_vib"].as_f64().unwrap() + 1.0e-2);
        assert!(!compare_json(&reference, &output, &cfg.required_fields(), &cfg.tolerance_overrides()));

        Ok(())
    }
}
