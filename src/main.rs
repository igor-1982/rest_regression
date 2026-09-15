use clap::{Arg, ArgMatches, Command};
use anyhow;
use anyhow::Context;
use regex::Regex;
//use core::num;
use std::{collections::HashMap, env, fs::{self, File}, io::{Read, Write}, path::Path, sync::mpsc};
use std::process::Command as ProcessCommand;
use std::time::Duration;

use walkdir::WalkDir;
use chrono::{DateTime, Local};

mod parse_grad;

mod filter;

mod json_ref;

pub fn parse_input() -> ArgMatches {
    Command::new("rest_regression")
        .version("0.1")
        .author("Igor Ying Zhang <igor_zhangying@fudan.edu.cn>")
        .about("A tool to perform regression tests for REST")
        .arg(Arg::new("regression_directory")
             .short('r')
             .long("regression_directory")
             //.default_value("$REST_HOME/utilities/rest_regression/bench_pool")
             .help("The directory that contains selected regression tasks [default: $REST_HOME/rest_regression/bench_pool]")
        )
        .arg(Arg::new("working_directory")
             .short('w')
             .long("working_directory")
             .default_value("./work_pool")
             .help("The working directory to store the regression output files")
        )
        .arg(Arg::new("rest_mode")
             .short('c')
             .long("rest_mode")
             .default_value("release")
             .help("The version of REST binary to invoke: \"release\" or \"debug\" ")
        )
        .arg(Arg::new("rest_path")
             .short('p')
             .long("rest_path")
             .help("The absolute path to find the `rest` binary [default: $REST_HOME/target/`rest_mode`/rest]")
        )
        .arg(Arg::new("n_mpi")
             .short('n')
             .long("n_mpi")
             .default_value("1")
             .help("The number of mpi tasks")
        )
        .arg(Arg::new("num_threads")
             .short('t')
             .long("num_threads")
             .help("The number of threads (overrides ctrl.in settings, ignored for special tests)")
        )
        .arg(Arg::new("filter")
             .short('f')
             .long("filter")
             .help("Comma-separated list of test names to run (e.g. \"H2O_guessfile,NH3_SCAN\")")
        )
        .arg(Arg::new("skip")
             .long("skip")
             .conflicts_with("filter")
             .help("Comma-separated list of test names to skip (e.g. \"NH3_SCAN,NH3_SVWN\")")
        )
        .arg(Arg::new("durations")
             .long("durations")
             .num_args(0..=1)
             .default_missing_value("5")
             .help("Show the slowest N tests [default: 5, use 0/off/false to disable]")
        )
        .arg(Arg::new("dry_run")
              .long("dry-run")
              .action(clap::ArgAction::SetTrue)
              .help("List tests that would be run without executing them")
          )
          .arg(Arg::new("timeout")
              .long("timeout")
              .help("Per-job timeout in seconds (0 = no timeout, default: no timeout)")
          )
          .arg(Arg::new("scalapack")
              .long("scalapack")
              .action(clap::ArgAction::SetTrue)
              .help("Force-enable the ScaLAPACK (distributed) paths: injects hamiltonian_distributed = \"on\" into [ctrl] and distributed = \"on\" into [ctrl.j2c_decomp] of every ctrl.in (or rest_jobs input) before running. Requires --n_mpi >= 2 (a warning is printed otherwise).")
          )
        //.arg(Arg::new("task_status")
        //     .short('s')
        //     .long("task_status")
        //     .help("The status of the task to be tested")
        //)
        .get_matches()
}

pub fn is_file_exist_in_dir(dir_name: &Path, file_name: &str) -> anyhow::Result<bool> {
    let mut is_exist = false;
    if !dir_name.exists() || !dir_name.is_dir() {
        return Ok(false);
    }
    for entry in fs::read_dir(dir_name)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.ends_with(file_name) {
            is_exist = true;
            break;
        }
    }
    Ok(is_exist)
}

fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub fn collect_results(file_name: &str) -> anyhow::Result<HashMap<String, Vec<f64>>> {
    let mut result = HashMap::new();

    let mut file = File::open(file_name).unwrap();

    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    let re_scf_eny = Regex::new(
      r"The SCF energy\s*:\s*(?P<energy>[\+-]?\d+.\d+)\s*"
    ).unwrap();

    if re_scf_eny.is_match(&buf) {
        let caps = re_scf_eny.captures(&buf).unwrap();
        let energy = caps.name("energy").unwrap().as_str().parse::<f64>().unwrap();
        result.insert("scf_energy".to_string(), vec![energy]);
    }

    // Solvent energy (electrostatic + SMD CDS). Printed only for solvent jobs,
    // so this key simply does not appear for gas-phase tests.
    let re_solvent_eny = Regex::new(
      r"The solvent energy\s*:\s*(?P<energy>[\+-]?\d+.\d+)\s*"
    ).unwrap();

    if re_solvent_eny.is_match(&buf) {
        let caps = re_solvent_eny.captures(&buf).unwrap();
        let energy = caps.name("energy").unwrap().as_str().parse::<f64>().unwrap();
        result.insert("solvent_energy".to_string(), vec![energy]);
    }

    let re_post_scf_eny = Regex::new(
      r"The \(R\)-xDH energy\s*:\s*(?P<energy>[\+-]?\d+.\d+)\s*"
    ).unwrap();

    if re_post_scf_eny.is_match(&buf) {
        let caps = re_post_scf_eny.captures(&buf).unwrap();
        let energy = caps.name("energy").unwrap().as_str().parse::<f64>().unwrap();
        result.insert("(R)-xDH energy".to_string(), vec![energy]);
    }

    let re_rpa_eny = Regex::new(
      r"The RPA energy\s*:\s*(?P<energy>[\+-]?\d+.\d+)\s*"
    ).unwrap();

    if re_rpa_eny.is_match(&buf) {
        let caps = re_rpa_eny.captures(&buf).unwrap();
        let energy = caps.name("energy").unwrap().as_str().parse::<f64>().unwrap();
        result.insert("RPA energy".to_string(), vec![energy]);
    }

    let re_dipole = Regex::new(
      r"Dipole Moment in DEBYE:\s*(?P<dx>[\+-]?\d+.\d+),\s*(?P<dy>[\+-]?\d+.\d+),\s*(?P<dz>[\+-]?\d+.\d+)"
    ).unwrap();

    if re_dipole.is_match(&buf) {
        let caps = re_dipole.captures(&buf).unwrap();
        let mut dp = vec![];
        dp.push(caps.name("dx").unwrap().as_str().parse::<f64>().unwrap());
        dp.push(caps.name("dy").unwrap().as_str().parse::<f64>().unwrap());
        dp.push(caps.name("dz").unwrap().as_str().parse::<f64>().unwrap());
        result.insert("Dipole moment".to_string(), dp);
    }

    let re_eps = Regex::new(
        r"RMSDs\sbetween\s\(ECP, ENXC\)\sand\s\(ECP, GEP\):\s\(\s*(?P<rmsd1>\d+.\d+),\s*(?P<rmsd2>\d+.\d+)\)"
    ).unwrap();    

    for cap in re_eps.captures_iter(&buf) {
        let rmsd1 = cap["rmsd1"].parse::<f64>().unwrap();
        let rmsd2 = cap["rmsd2"].parse::<f64>().unwrap();
        result.insert("EP Benchmark".to_string(), vec![rmsd1, rmsd2]);
    }
    let re_gw_homo=Regex::new(
        r"The\sQP\senergy\sof\sHOMO\sobtained\sby\sGWA\sis\s*(?P<gw_homo>[\+-]?\d+\.\d+)"
    ).unwrap();

    if re_gw_homo.is_match(&buf) {
        let caps = re_gw_homo.captures(&buf).unwrap();
        let energy = caps.name("gw_homo").unwrap().as_str().parse::<f64>().unwrap();
        result.insert("GW HOMO QP".to_string(), vec![energy]);
    }

    let re_gw_lumo=Regex::new(
        r"The\sQP\senergy\sof\sLUMO\sobtained\sby\sGWA\sis\s*(?P<gw_lumo>[\+-]?\d+\.\d+)"
    ).unwrap();

    if re_gw_lumo.is_match(&buf) {
        let caps = re_gw_lumo.captures(&buf).unwrap();
        let energy = caps.name("gw_lumo").unwrap().as_str().parse::<f64>().unwrap();
        result.insert("GW LUMO QP".to_string(), vec![energy]);
    }

    let re_bse=Regex::new(
        r"The\sfirst\sexcitation\sobtained\sby\sBSE\sis\s*(?P<bse>[\+-]?\d+\.\d+)"
    ).unwrap();

    if re_bse.is_match(&buf) {
        let caps = re_bse.captures(&buf).unwrap();
        let energy = caps.name("bse").unwrap().as_str().parse::<f64>().unwrap();
        result.insert("BSE 1st Excitation".to_string(), vec![energy]);
    }
    let re_grad = Regex::new(
        r"[-]{3,} *Output gradient *\[[a-zA-Z.]*\] *[-]{3,}\n+(?P<grad>[\S\s]*)\n+[-]{3,}"
    ).unwrap();

    if re_grad.is_match(&buf) {
        let caps = re_grad.captures(&buf).unwrap();
        let grad = crate::parse_grad::parse_grad(&caps["grad"].to_string());
        result.insert("gradient".to_string(), grad);
    }

    // ── Hessian regression ──
    let re_hess_total = Regex::new(
        r"Hessian total max_abs\s*=\s*(?P<val>[\+-]?\d+\.\d+e[+-]?\d+)"
    ).unwrap();
    if re_hess_total.is_match(&buf) {
        let caps = re_hess_total.captures(&buf).unwrap();
        let val = caps["val"].parse::<f64>().unwrap();
        result.insert("hessian_total_max_abs".to_string(), vec![val]);
    }

    let re_hess_hpart = Regex::new(
        r"h_partial max_abs\s*=\s*(?P<val>[\+-]?\d+\.\d+e[+-]?\d+)"
    ).unwrap();
    if re_hess_hpart.is_match(&buf) {
        let caps = re_hess_hpart.captures(&buf).unwrap();
        let val = caps["val"].parse::<f64>().unwrap();
        result.insert("h_partial_max_abs".to_string(), vec![val]);
    }

    let re_hess_cphf = Regex::new(
        r"cphf_contrib max_abs\s*=\s*(?P<val>[\+-]?\d+\.\d+e[+-]?\d+)"
    ).unwrap();
    if re_hess_cphf.is_match(&buf) {
        let caps = re_hess_cphf.captures(&buf).unwrap();
        let val = caps["val"].parse::<f64>().unwrap();
        result.insert("cphf_contrib_max_abs".to_string(), vec![val]);
    }

    // ── Frequencies regression ──
    // Parse the stdout frequency table: "  N    FREQ    ---" format
    //   Vibrational Frequencies (cm):
    //   Mode     Freq/cm    Symmetry
    //   ----     ---------- ----------
    //    1       -601.0371  ---
    let re_freq = Regex::new(
        r"(?m)^\s*(?P<idx>\d+)\s+(?P<freq>[\+-]?\d+\.\d+)\s+(?:---|[A-Z])"
    ).unwrap();
    let mut freqs: Vec<f64> = vec![];
    for cap in re_freq.captures_iter(&buf) {
        freqs.push(cap["freq"].parse::<f64>().unwrap());
    }
    if !freqs.is_empty() {
        result.insert("frequencies_cm".to_string(), freqs);
    }

    // ── Geometry optimization: final bond length (diatomic) ──
    // "Geometry after relaxation [Ang]:" block contains atomic coordinates.
    // For diatomic molecules, compute the distance between atom 1 and atom 2.
    let re_geom = Regex::new(
        r"Geometry after relaxation \[Ang\]:\n(?P<atom1>[A-Z][a-z]?)\s+(?P<x1>[\+-]?\d+\.\d+)\s+(?P<y1>[\+-]?\d+\.\d+)\s+(?P<z1>[\+-]?\d+\.\d+)\s*\n(?P<atom2>[A-Z][a-z]?)\s+(?P<x2>[\+-]?\d+\.\d+)\s+(?P<y2>[\+-]?\d+\.\d+)\s+(?P<z2>[\+-]?\d+\.\d+)"
    ).unwrap();

    if let Some(caps) = re_geom.captures(&buf) {
        let x1 = caps["x1"].parse::<f64>().unwrap();
        let y1 = caps["y1"].parse::<f64>().unwrap();
        let z1 = caps["z1"].parse::<f64>().unwrap();
        let x2 = caps["x2"].parse::<f64>().unwrap();
        let y2 = caps["y2"].parse::<f64>().unwrap();
        let z2 = caps["z2"].parse::<f64>().unwrap();
        let dx = x2 - x1;
        let dy = y2 - y1;
        let dz = z2 - z1;
        let bond_length = (dx*dx + dy*dy + dz*dz).sqrt();
        result.insert("bond_length".to_string(), vec![bond_length]);
    }

    let re_hoco_luco_gap = Regex::new(
        r"HOCO-LUCO gap\s*:\s*(?P<energy>[-+]?\d+\.\d+)\s*eV"
        ).unwrap();

    if re_hoco_luco_gap.is_match(&buf) {
        let caps = re_hoco_luco_gap.captures(&buf).unwrap();
        let gap_energy = caps.name("energy").unwrap().as_str().parse::<f64>().unwrap();
        result.insert("hoco_luco_gap".to_string(),vec![gap_energy]);
    }

    // TDDFT excitation energies: extract first 3 excitation energies
    // New BSE-like format from REST:
    // First 3 Singlet Excitations:
    // #0 Excitation energy=0.11186693, norm=1.000000
    // 	Transition Dipole Square:0.XXXX; Oscillator Strength:0.XXXX
    let re_tddft_energy = Regex::new(
        r"(?m)^#(?P<idx>\d+) Excitation energy=(?P<energy>[\+-]?\d+\.\d+)"
    ).unwrap();
    let re_tddft_osc = Regex::new(
        r"(?m)Oscillator Strength:(?P<osc>[\+-]?\d+\.\d+)"
    ).unwrap();

    let mut tddft_energies: Vec<f64> = vec![];
    let mut tddft_osc: Vec<f64> = vec![];
    for cap in re_tddft_energy.captures_iter(&buf) {
        let idx = cap["idx"].parse::<usize>().unwrap_or(99);
        if idx <= 6 {
            tddft_energies.push(cap["energy"].parse::<f64>().unwrap());
        }
    }
    for cap in re_tddft_osc.captures_iter(&buf) {
        tddft_osc.push(cap["osc"].parse::<f64>().unwrap());
    }
    if !tddft_energies.is_empty() {
        result.insert("tddft_energies".to_string(), tddft_energies);
        result.insert("tddft_osc".to_string(), tddft_osc);
    }

    Ok(result)
}

pub fn compare_results(ref_hashmap: &HashMap<String, Vec<f64>>, out_hashmap: &HashMap<String, Vec<f64>>) -> bool {
    let mut is_same = true;
    println!("                                     Current Value vs      Reference Value");
    for (key, ref_value) in ref_hashmap.iter() {
        if key.eq("Dipole moment") {
            if let Some(out_value) = out_hashmap.get(key) {
                let dev = out_value.iter().zip(ref_value.iter()).fold(0.0, |r, (o, f)| {
                    r + (*o- *f).powf(2.0)
                }).powf(0.5);

                if dev > 1.0e-4 {
                    is_same = false;
                    println!("WARNNING: {:20}:{:?} != {:?}", key, out_value, ref_value);
                } else {
                    println!("Pass    : {:20}:{:?}  = {:?}", key, out_value, ref_value);
                }
            } else {
                is_same = false;
                println!("{:20} not found in output", key);
                //break;
            }
        } else if key.eq("gradient") {
            if let Some(out_value) = out_hashmap.get(key) {
                let out_norm = out_value.iter().fold(0.0, |r, f| r + f.powf(2.0)).powf(0.5);
                let ref_norm = ref_value.iter().fold(0.0, |r, f| r + f.powf(2.0)).powf(0.5);
                let dev = out_value.iter().zip(ref_value.iter()).fold(0.0, |r, (o, f)| {
                    r + (*o - *f).powf(2.0)
                }).powf(0.5);

                if dev > 1.0e-4 {
                    is_same = false;
                    println!("WARNNING: {:20}: norm {:20.10} != norm {:20.10}", key, out_norm, ref_norm);
                } else {
                    println!("Pass    : {:20}: norm {:20.10}  = norm {:20.10}", key, out_norm, ref_norm);
                }
            } else {
                is_same = false;
                println!("{:20} not found in output", key);
                //break;
            }
            
        } else if key.eq("hessian_total_max_abs") {
            if let Some(out_value) = out_hashmap.get(key) {
                let dev = (ref_value[0] - out_value[0]).abs();
                if dev > 1.0e-5 {
                    is_same = false;
                    println!("WARNNING: {:20}:{:20.10} != {:20.10} (dev={:.2e})", key, out_value[0], ref_value[0], dev);
                } else {
                    println!("Pass    : {:20}:{:20.10}  = {:20.10} (dev={:.2e})", key, out_value[0], ref_value[0], dev);
                }
            } else {
                is_same = false;
                println!("{:20} not found in output", key);
            }
        } else if key.eq("h_partial_max_abs") || key.eq("cphf_contrib_max_abs") {
            if let Some(out_value) = out_hashmap.get(key) {
                let dev = (ref_value[0] - out_value[0]).abs();
                if dev > 1.0e-4 {
                    is_same = false;
                    println!("WARNNING: {:20}:{:20.10} != {:20.10} (dev={:.2e})", key, out_value[0], ref_value[0], dev);
                } else {
                    println!("Pass    : {:20}:{:20.10}  = {:20.10} (dev={:.2e})", key, out_value[0], ref_value[0], dev);
                }
            } else {
                is_same = false;
                println!("{:20} not found in output", key);
            }
        } else if key.eq("frequencies_cm") {
            if let Some(out_value) = out_hashmap.get(key) {
                let n_modes = ref_value.len().min(out_value.len());
                for i in 0..n_modes {
                    let dev = (ref_value[i] - out_value[i]).abs();
                    if dev > 2.0 {
                        is_same = false;
                        println!("WARNNING: Frequency Mode {}: {:12.4} != {:12.4} (dev={:.2e} cm⁻¹)",
                            i + 1, out_value[i], ref_value[i], dev);
                    } else {
                        println!("Pass    : Frequency Mode {}: {:12.4}  = {:12.4} (dev={:.2e} cm⁻¹)",
                            i + 1, out_value[i], ref_value[i], dev);
                    }
                }
            } else {
                is_same = false;
                println!("{:20} not found in output", key);
            }
        } else if key.eq("hoco_luco_gap") {
            if let Some(out_value) = out_hashmap.get(key) {
                if (ref_value[0] - out_value[0]).abs() > 1e-5 {
                    is_same = false;
                    println!("WARNNING: {:20}:{:20.10} != {:20.10}", key, out_value[0], ref_value[0]);
                } else {
                    println!("Pass    : {:20}:{:20.10}  = {:20.10}", key, out_value[0], ref_value[0]);
                }
            } else {
                is_same = false;
                println!("{:20} not found in output", key);
            }
        } else if key.eq("bond_length") {
            if let Some(out_value) = out_hashmap.get(key) {
                let dev = (ref_value[0] - out_value[0]).abs();
                if dev > 5.0e-4 {
                    is_same = false;
                    println!("WARNNING: {:20}:{:12.8} != {:12.8} (dev={:.2e} Ang)",
                        key, out_value[0], ref_value[0], dev);
                } else {
                    println!("Pass    : {:20}:{:12.8}  = {:12.8} (dev={:.2e} Ang)",
                        key, out_value[0], ref_value[0], dev);
                }
            } else {
                is_same = false;
                println!("{:20} not found in output", key);
            }
        } else if key.eq("tddft_energies") {
            if let Some(out_value) = out_hashmap.get(key) {
                let n_states = ref_value.len().min(out_value.len());
                for i in 0..n_states {
                    let dev = (ref_value[i] - out_value[i]).abs();
                    if dev > 5.0e-5 {
                        is_same = false;
                        println!("WARNNING: TDDFT State {}: {:20.10} != {:20.10} (dev={:.2e})",
                            i+1, out_value[i], ref_value[i], dev);
                    } else {
                        println!("Pass    : TDDFT State {}: {:20.10}  = {:20.10} (dev={:.2e})",
                            i+1, out_value[i], ref_value[i], dev);
                    }
                }
            } else {
                is_same = false;
                println!("{:20} not found in output", key);
            }
        } else if key.eq("tddft_osc") {
            if let Some(out_value) = out_hashmap.get(key) {
                let n_states = ref_value.len().min(out_value.len());
                for i in 0..n_states {
                    let dev = (ref_value[i] - out_value[i]).abs();
                    if dev > 1.0e-4 {
                        is_same = false;
                        println!("WARNNING: TDDFT Osc {}: {:20.10} != {:20.10} (dev={:.2e})",
                            i+1, out_value[i], ref_value[i], dev);
                    } else {
                        println!("Pass    : TDDFT Osc {}: {:20.10}  = {:20.10} (dev={:.2e})",
                            i+1, out_value[i], ref_value[i], dev);
                    }
                }
            } else {
                is_same = false;
                println!("{:20} not found in output", key);
            }
        } else { 
            if let Some(out_value) = out_hashmap.get(key) {
                if (ref_value[0] - out_value[0]).abs() > 1e-5 {
                    is_same = false;
                    println!("WARNNING: {:20}:{:20.10} != {:20.10}", key, out_value[0], ref_value[0]);
                    //break;
                } else {
                    println!("Pass    : {:20}:{:20.10}  = {:20.10}", key, out_value[0], ref_value[0]);
                }
            } else {
                is_same = false;
                println!("{:20} not found in output", key);
                //break;
            }
        }
    }
    is_same
}

pub fn init_timing() -> DateTime<Local> {
    Local::now()
}

pub fn timing(dt0: &DateTime<Local>, iprint: Option<&str>) -> DateTime<Local> {
    let dt1 = Local::now();
    match iprint {
        None => {dt1},
        Some(header) => {
            let timecost1 = (dt1.timestamp_millis()-dt0.timestamp_millis()) as f64 /1000.0;
            println!("{:30} cost {:6.2} seconds", header, timecost1);
            dt1
        }                                                                                                                                                            
    }
}

fn set_num_threads_in_file(file_path: &Path, num_threads: usize) -> anyhow::Result<()> {
    let content = fs::read_to_string(file_path)?;
    let re = Regex::new(r"(?m)^\s*num_threads\s*=\s*\d+").unwrap();
    let new_content = if re.is_match(&content) {
        re.replace(&content, format!("num_threads = {}", num_threads)).to_string()
    } else {
        let re_ctrl = Regex::new(r"(?m)^\[ctrl\]").unwrap();
        if re_ctrl.is_match(&content) {
            re_ctrl.replace(&content, format!("[ctrl]\nnum_threads = {}", num_threads)).to_string()
        } else {
            content
        }
    };
    let mut f = File::create(file_path)?;
    f.write_all(new_content.as_bytes())?;
    Ok(())
}

/// Force-enable the ScaLAPACK (distributed) paths in a ctrl.in file:
///   - `hamiltonian_distributed = "on"` into the `[ctrl]` section,
///   - `distributed = "on"` into the `[ctrl.j2c_decomp]` section
///     (creating the section when absent, appended at the end of the file).
fn set_scalapack_in_file(file_path: &Path) -> anyhow::Result<()> {
    let content = fs::read_to_string(file_path)?;

    // Geometry-optimization jobs (job_type = "opt") are excluded from the ScaLAPACK
    // injection: their geomeTRIC master/slave MPI loop with repeated force evaluations
    // intermittently hangs under injected distributed diagonalization when run in the
    // fully concurrent suite (CO_OPT_LBFGS_NumForce exceeded the 1800 s timeout while
    // the same task passes solo in ~13 s and always passes in the plain --mpi gate).
    // Distributed linear algebra coverage is provided by the single-point / gradient
    // tasks instead.
    if content.lines().any(|l| l.trim().to_lowercase().starts_with("job_type") && l.contains("opt")) {
        println!("Skipping ScaLAPACK injection for geometry-optimization job: {}", file_path.display());
        return Ok(());
    }

    let mut lines: Vec<&str> = content.lines().collect();

    // 1) inject hamiltonian_distributed into [ctrl] (first key line of the section)
    if let Some(ctrl_idx) = lines.iter().position(|l| l.trim() == "[ctrl]") {
        let mut insert_at: Option<usize> = None;
        for (i, l) in lines.iter().enumerate().skip(ctrl_idx + 1) {
            let t = l.trim();
            if t.starts_with('[') {
                break; // next section header
            }
            if t.is_empty() || t.starts_with('#') || t.starts_with("//") {
                continue;
            }
            if t.starts_with("hamiltonian_distributed") {
                insert_at = None; // already present
                break;
            }
            insert_at = Some(i);
            break;
        }
        if let Some(idx) = insert_at {
            lines.insert(idx, "hamiltonian_distributed = \"on\"");
        }
    } else {
        lines.insert(0, "hamiltonian_distributed = \"on\"");
        lines.insert(0, "[ctrl]");
    }

    // 1b) inject rpa_distributed into [ctrl] (same section; skip if hamiltonian_distributed
    //     was already present and inserted right before it)
    if let Some(ctrl_idx) = lines.iter().position(|l| l.trim() == "[ctrl]") {
        let mut insert_at: Option<usize> = None;
        for (i, l) in lines.iter().enumerate().skip(ctrl_idx + 1) {
            let t = l.trim();
            if t.starts_with('[') {
                break;
            }
            if t.is_empty() || t.starts_with('#') || t.starts_with("//") {
                continue;
            }
            if t.starts_with("rpa_distributed") {
                insert_at = None;
                break;
            }
            insert_at = Some(i);
            break;
        }
        if let Some(idx) = insert_at {
            lines.insert(idx, "rpa_distributed = \"on\"");
        }
    }

    // 2) inject distributed into [ctrl.j2c_decomp]
    let j2c_section = lines.iter().position(|l| l.trim() == "[ctrl.j2c_decomp]");
    match j2c_section {
        Some(j2c_idx) => {
            let mut insert_at: Option<usize> = None;
            for (i, l) in lines.iter().enumerate().skip(j2c_idx + 1) {
                let t = l.trim();
                if t.starts_with('[') {
                    break;
                }
                if t.is_empty() || t.starts_with('#') || t.starts_with("//") {
                    continue;
                }
                if t.starts_with("distributed") {
                    insert_at = None;
                    break;
                }
                insert_at = Some(i);
                break;
            }
            if let Some(idx) = insert_at {
                lines.insert(idx, "distributed = \"on\"");
            }
        }
        None => {
            // append a new section at the end of the file
            lines.push("");
            lines.push("[ctrl.j2c_decomp]");
            lines.push("distributed = \"on\"");
        }
    }

    let mut new_content = lines.join("\n");
    new_content.push('\n');
    let mut f = File::create(file_path)?;
    f.write_all(new_content.as_bytes())?;
    Ok(())
}

fn main() -> anyhow::Result<()> {

    let s_time = init_timing();

    let mut is_pass = true;

    //let current_directory = env::current_dir().unwrap().as_os_str().to_os_string().to_str().unwrap().to_string();
    let current_directory = env::current_dir().unwrap().to_str().unwrap().to_string();
    let curr_path = Path::new(&current_directory);
    

    let rest_home = env::var_os("REST_HOME").map(|v| v.to_string_lossy().to_string());

    let input_folder = parse_input().get_one::<String>("regression_directory")
        .cloned()
        .unwrap_or_else(|| {
            let local = Path::new("./bench_pool");
            if local.is_dir() {
                return local.to_string_lossy().to_string();
            }
            let home = rest_home.as_ref()
                .expect("REST_HOME not defined. Either set REST_HOME, provide -r/--regression_directory, or create ./bench_pool.");
            format!("{}/rest_regression/bench_pool", home)
        });
    
    let work_folder = parse_input().get_one::<String>("working_directory").unwrap().to_string();
    let work_path = Path::new(&work_folder);

    let num_mpi = parse_input().get_one::<String>("n_mpi").unwrap().parse::<usize>().unwrap();
    println!("num_mpi = {}", num_mpi);

    let num_threads: Option<usize> = parse_input().get_one::<String>("num_threads")
        .map(|s| s.parse::<usize>().expect("num_threads must be a positive integer"));

    let global_timeout: Option<u64> = parse_input().get_one::<String>("timeout")
        .map(|s| s.parse::<u64>().expect("timeout must be a positive integer"))
        .filter(|&t| t > 0);

    let filter_set: Option<std::collections::HashSet<String>> = {
        let matches = parse_input();
        matches.get_one::<String>("filter").map(|s| {
            s.split(',').map(|n| n.trim().to_lowercase()).collect()
        })
    };
    let mut filter_unmatched = filter_set.clone();

    if let Some(ref set) = filter_set {
        println!("Filter: running only {:?}", set.iter().collect::<Vec<_>>());
    }

    let skip_set: Option<std::collections::HashSet<String>> = {
        let matches = parse_input();
        matches.get_one::<String>("skip").map(|s| {
            s.split(',').map(|n| n.trim().to_lowercase()).collect()
        })
    };
    let mut skip_unmatched = skip_set.clone();

    if let Some(ref set) = skip_set {
        println!("Skip: skipping {:?}", set.iter().collect::<Vec<_>>());
    }

    let durations_n: usize = {
        let matches = parse_input();
        if matches.contains_id("durations") {
            let val = matches.get_one::<String>("durations").map(|s| s.to_lowercase()).unwrap_or_default();
            match val.as_str() {
                "0" | "off" | "false" => 0,
                _ => val.parse::<usize>().expect("--durations must be a positive integer, 0, off, or false"),
            }
        } else {
            5
        }
    };
    let mut durations: Vec<(String, f64)> = vec![];
    let dry_run: bool = parse_input().get_flag("dry_run");
    let use_scalapack: bool = parse_input().get_flag("scalapack");
    if use_scalapack && num_mpi < 2 {
        println!("WARNING: --scalapack forces the distributed (ScaLAPACK) paths, which require MPI.");
        println!("         --n_mpi is {} (< 2); the injected keywords will have no effect.", num_mpi);
    }

    let test_config = filter::load_test_config();
    let all_names = filter::build_all_names(&input_folder, &test_config);

    if let Some(ref mut unmatched) = filter_unmatched {
        unmatched.retain(|n| all_names.contains(n));
    }
    if let Some(ref mut unmatched) = skip_unmatched {
        unmatched.retain(|n| all_names.contains(n));
    }


    fs::create_dir(work_path).unwrap_or(());
    //fs::create_dir(work_path).unwrap_err();

    let rest_cmd = {
        let from_cli = parse_input().get_one::<String>("rest_path").cloned();
        from_cli.unwrap_or_else(|| {
            let rest_mode = parse_input().get_one::<String>("rest_mode").unwrap().to_string();
            let home = rest_home.as_ref()
                .expect("REST_HOME not defined. Either set REST_HOME or provide -p/--rest_path.");
            format!("{}/target/{}/rest", home, rest_mode)
        })
    };
    if ! std::path::Path::new(&rest_cmd).is_file() {
        panic!("The REST binary \'{}\' does not exist", &rest_cmd);
    }

    println!("REST: {}", &rest_cmd);



    let mut job_index = 0;
    let mut fail_list = vec![];
    let mut njob: usize = 0;
    let mut npass: usize = 0;
    let mut nfail: usize = 0;
    
    for entry in WalkDir::new(&input_folder)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        .into_iter().filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir() 
            && is_file_exist_in_dir(e.path(), &"reference.log").unwrap()
            //&& is_file_exist_in_dir(e.path(), &"ctrl.in").unwrap()
        ) {

        if let Some(ref set) = filter_set {
            if !filter::entry_matches(&entry, &input_folder, set, &test_config) {
                continue;
            }
            if let Some(ref mut unmatched) = filter_unmatched {
                filter::remove_matched(&entry, &input_folder, set, &test_config, unmatched);
            }
        }

        if let Some(ref set) = skip_set {
            if filter::entry_matches(&entry, &input_folder, set, &test_config) {
                if let Some(ref mut unmatched) = skip_unmatched {
                    filter::remove_matched(&entry, &input_folder, set, &test_config, unmatched);
                }
                continue;
            }
        }

        if dry_run {
            println!("Would run: {}", entry.path().display());
            njob += 1;
            job_index += 1;
            continue;
        }

        println!("====================================================");
        println!("Start regression task: {}", entry.path().display());
        let j_time = init_timing();


        // at first, collect the reference results
        // a config.toml in the test folder can pin the reference mode and tune the
        // json comparison; by default ("auto") the test is compared against
        // reference.json when that file is present, and by log scraping otherwise
        let test_cfg = json_ref::load_per_test_config(entry.path())?;
        let ref_mode = match &test_cfg {
            Some(cfg) => cfg.reference_mode(entry.path()),
            None => json_ref::ReferenceMode::auto(entry.path()),
        };
        let ref_log = format!("{}/reference.log", entry.path().to_str().unwrap());
        let (ref_hashmap, ref_json): (Option<HashMap<String, Vec<f64>>>, Option<serde_json::Value>) = match ref_mode {
            json_ref::ReferenceMode::Json => {
                let explicit_json = matches!(&test_cfg, Some(cfg) if cfg.declared_reference().as_deref() == Some("json"));
                let how = if explicit_json {
                    "declares reference = \"json\" in config.toml"
                } else {
                    "auto-detected reference.json in the test folder"
                };
                println!("Reference mode: json ({})", how);
                let v = json_ref::load_json_file(&entry.path().join("reference.json"))
                    .with_context(|| format!("test '{}' {}, \
                                             but reference.json is missing or unreadable", entry.path().display(), how))?;
                (None, Some(v))
            }
            json_ref::ReferenceMode::Log => {
                if test_cfg.as_ref().is_some_and(|cfg| !cfg.required_fields().is_empty()) {
                    println!("WARNING : config.toml carries fields/tolerance, which only apply to json-reference mode; \
                              this test is compared in log mode");
                }
                (Some(collect_results(&ref_log)?), None)
            }
        };
        

        // print "README" if exist
        if is_file_exist_in_dir(entry.path(), &"README").unwrap() {
            for line in fs::read_to_string(entry.path().join("README")).unwrap().lines() {
                println!("{}", line);
            }
        }


        // initialize the output file
        let file_name = entry.path().file_name().unwrap();
        let store_file = curr_path.join(&work_folder).join(file_name).with_extension("log");
        println!("output file: {}", store_file.display());

        // collect the job list, which will be performed in sequence
        let mut job_list = vec![];
        if is_file_exist_in_dir(entry.path(), "rest_jobs").unwrap() {
            for line in fs::read_to_string(entry.path().join("rest_jobs")).unwrap().lines() {
                if ! is_file_exist_in_dir(entry.path(), line).unwrap() {
                    panic!("{} not found in {}", line, entry.path().display());
                }
                job_list.push(line.to_string());
                println!("{}", line);
            }
        } else {
            if ! is_file_exist_in_dir(entry.path(), "ctrl.in").unwrap() {
                panic!("ctrl.in not found in {}", entry.path().display());
            }
            job_list.push("ctrl.in".to_string());
        }

        // absolute path: artifacts are copied out below while the cwd is still the job dir,
        // and work_path itself may be relative (default ./work_pool)
        let job_work_dir = curr_path.join(work_path).join(entry.path().file_name().unwrap());
        copy_dir_all(entry.path(), &job_work_dir)?;

        if ref_mode == json_ref::ReferenceMode::Json {
            // remove the copied reference files from the job dir, so that a stale reference.json
            // can never be mistaken for (or silently clobbered by) the REST output
            let _ = fs::remove_file(job_work_dir.join("reference.json"));
            let _ = fs::remove_file(job_work_dir.join("reference.log"));
        }

        let test_name = entry.path().file_name().unwrap().to_str().unwrap();
        let (effective_num_mpi, effective_num_threads, effective_timeout) = match test_config.get(test_name) {
            Some(config) if config.num_mpi.is_some() || config.num_threads.is_some() || config.timeout.is_some() => {
                println!("Special test config: num_mpi={:?}, num_threads={:?}, timeout={:?}", config.num_mpi, config.num_threads, config.timeout);
                let t = match config.timeout {
                    Some(0) | None => global_timeout,
                    Some(s) => Some(s),
                };
                (config.num_mpi.unwrap_or(num_mpi), config.num_threads, t)
            },
            _ => (num_mpi, num_threads, global_timeout),
        };
        let effective_duration = effective_timeout.map(Duration::from_secs);

        if let Some(nt) = effective_num_threads {
            for job_file_name in &job_list {
                let job_file_path = job_work_dir.join(job_file_name);
                if job_file_path.is_file() {
                    set_num_threads_in_file(&job_file_path, nt)?;
                    println!("Set num_threads = {} in {}", nt, job_file_name);
                }
            }
        }

        if use_scalapack {
            for job_file_name in &job_list {
                let job_file_path = job_work_dir.join(job_file_name);
                if job_file_path.is_file() {
                    set_scalapack_in_file(&job_file_path)?;
                    println!("Force-enabled ScaLAPACK keywords in {}", job_file_name);
                }
            }
        }

        // enter the job folder
        env::set_current_dir(&job_work_dir)?;

        for job in job_list {
            let log_file = File::create(&store_file)?;

            let (tx, rx) = mpsc::channel();
            let cmd_clone = rest_cmd.clone();
            let job_clone = job.clone();
            let n_mpi = effective_num_mpi;

            std::thread::spawn(move || {
                let result = if n_mpi == 1 {
                    ProcessCommand::new(&cmd_clone)
                        .stdout(log_file.try_clone().unwrap())
                        .stderr(log_file)
                        .arg("-i").arg(&job_clone).status()
                } else {
                    ProcessCommand::new("mpirun").arg("-n").arg(format!("{}", n_mpi))
                        .arg(&cmd_clone)
                        .stdout(log_file.try_clone().unwrap())
                        .stderr(log_file)
                        .arg("-i").arg(&job_clone).status()
                };
                let _ = tx.send(result);
            });

            let status = match effective_duration {
                Some(dur) => match rx.recv_timeout(dur) {
                    Ok(result) => result?,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        anyhow::bail!("timeout: job '{}' in test '{}' exceeded {} seconds",
                            job, test_name, dur.as_secs());
                    }
                    Err(_) => unreachable!(),
                },
                None => rx.recv().unwrap()?,
            };

            if !status.success() {
                anyhow::bail!("process '{}' in test '{}' exited with {}", job, test_name, status);
            }
        }
        // Save Hessian/Frequencies output files before leaving & deleting the job dir
        for artifact_name in &["HessianMatrix.txt", "EigenModes.txt"] {
            let src = job_work_dir.join(artifact_name);
            if src.exists() {
                let mut dst_name = test_name.to_string();
                dst_name.push('_');
                dst_name.push_str(artifact_name);
                let _ = fs::copy(&src, curr_path.join(work_path).join(&dst_name));
            }
        }

        // Save the json result file of a json-reference test before deleting the job dir
        let mut json_store_file: Option<std::path::PathBuf> = None;
        if ref_mode == json_ref::ReferenceMode::Json {
            let dst = curr_path.join(work_path).join(format!("{}.json", test_name));
            match fs::copy(job_work_dir.join("reference.json"), &dst) {
                Ok(_) => json_store_file = Some(dst),
                Err(_) => println!("WARNING : REST did not produce reference.json in the job dir; \
                                   check that ctrl.in sets outname = \"reference\""),
            }
        }

        // leave the job folder
        env::set_current_dir(curr_path)?;
        // remove the tmp job folder
        fs::remove_dir_all(&job_work_dir)?;

        let is_pass_current = match ref_mode {
            json_ref::ReferenceMode::Json => {
                let cfg = test_cfg.as_ref().unwrap();
                match (&ref_json, &json_store_file) {
                    (Some(r), Some(p)) => {
                        let out_json = json_ref::load_json_file(p)
                            .with_context(|| format!("cannot read the output json of test '{}'", test_name))?;
                        json_ref::compare_json(r, &out_json, &cfg.required_fields(), &cfg.tolerance_overrides())
                    }
                    _ => {
                        println!("WARNING : test '{}' failed: the output reference.json is missing", test_name);
                        false
                    }
                }
            }
            json_ref::ReferenceMode::Log => {
                let out_hashmap = collect_results(store_file.to_str().unwrap())?;
                compare_results(ref_hashmap.as_ref().unwrap(), &out_hashmap)
            }
        };
        
        njob += 1;
        if ! is_pass_current {
            fail_list.push(entry.path().display().to_string());
            nfail += 1;
        } else {
            npass += 1;
        }

        is_pass = is_pass && is_pass_current;

        let dt1 = timing(&j_time, Some(&format!("Job {:3}", job_index)));
        if durations_n > 0 {
            let elapsed = (dt1.timestamp_millis() - j_time.timestamp_millis()) as f64 / 1000.0;
            durations.push((test_name.to_string(), elapsed));
        }
        job_index += 1;

    }

    if let Some(ref unmatched) = filter_unmatched {
        if !unmatched.is_empty() {
            panic!("Filtered tests not found: {:?}", unmatched.iter().collect::<Vec<_>>());
        }
    }

    if let Some(ref unmatched) = skip_unmatched {
        if !unmatched.is_empty() {
            panic!("Skipped tests not found: {:?}", unmatched.iter().collect::<Vec<_>>());
        }
    }

    if durations_n > 0 {
        durations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let n = durations_n.min(durations.len());
        if n > 0 {
            println!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
            println!("Slowest {} tests:", n);
            for (i, (name, secs)) in durations.iter().take(n).enumerate() {
                println!("{:3}: {:30} {:8.2}s", i + 1, name, secs);
            }
        }
    }

    println!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    if dry_run {
        println!("Dry run: {} tests would be run", njob);
    } else {
        println!("Regression summary: {}/{} passed, {}/{} failed", npass, njob, nfail, njob);
        if is_pass {
            println!("All regression tasks passed");
        } else {
            println!("Regression failed for");
            for job in fail_list {
                println!("X: \"{}\"", job);
            }
        }
    }
    println!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

    timing(&s_time, Some(&"Total time consumption"));

    if is_pass {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Some regression tasks failed"))
    }
}

#[test]
pub fn test_env() -> anyhow::Result<()>{
    use std::env;
    let key = "REST_HOME";
    match env::var_os(key) {
        Some(val) => {
            let mut val2 = val.clone();
            val2.push("/utilities/rest_regression/bench_pools");
            //let val3 = format!("{:?}/utilities/bench_pools",val2);
            println!("{}: {:?}", key, val2);
            //for entry in fs::read_dir(format!("{:?}/utilities/bench_pools",val))? {
            for entry in fs::read_dir(val2)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    println!("Directory: {}", path.display());
                } else {
                    println!("File: {}", path.display());
                }
            }
        },
        None => println!("{} not defined in the environment.", key),
    }

    Ok(())
}

#[test]
pub fn test_tddft_collect_results() -> anyhow::Result<()> {
    // Use CARGO_MANIFEST_DIR (compile-time constant) to locate bench_pool regardless of
    // where cargo test is invoked from.
    let bench_pool = Path::new(env!("CARGO_MANIFEST_DIR")).join("bench_pool");

    // Test each TDDFT reference log
    for test_name in &["H2_PBE0_TDA", "H2_PBE_TDA", "H2_SVWN_TDA"] {
        let ref_log = bench_pool.join("tddft").join(test_name).join("reference.log");
        let ref_log_str = ref_log.to_str().unwrap();
        println!("\n=== Testing {} ===", test_name);
        let result = collect_results(ref_log_str)?;

        // Check scf_energy is extracted
        assert!(result.contains_key("scf_energy"),
            "{}: scf_energy not found in reference.log", test_name);
        println!("  scf_energy = {:?}", result.get("scf_energy"));

        // Check tddft_energies is extracted
        assert!(result.contains_key("tddft_energies"),
            "{}: tddft_energies not found in reference.log", test_name);
        let energies = result.get("tddft_energies").unwrap();
        assert_eq!(energies.len(), 3,
            "{}: expected 3 excitation energies, got {}", test_name, energies.len());
        println!("  tddft_energies = {:?}", energies);

        // Check tddft_osc is extracted
        assert!(result.contains_key("tddft_osc"),
            "{}: tddft_osc not found in reference.log", test_name);
        let osc = result.get("tddft_osc").unwrap();
        assert_eq!(osc.len(), 3,
            "{}: expected 3 oscillator strengths, got {}", test_name, osc.len());
        println!("  tddft_osc = {:?}", osc);

        // Verify energy values look reasonable (between 0 and 2 Ha for valence excitations)
        for (i, &e) in energies.iter().enumerate() {
            assert!(e > 0.0 && e < 2.0,
                "{}: excitation energy {} (state {}) out of range [0, 2]", test_name, e, i);
        }

        // Verify oscillator strengths are non-negative
        for (i, &o) in osc.iter().enumerate() {
            assert!(o >= 0.0,
                "{}: oscillator strength {} (state {}) is negative", test_name, o, i);
        }
    }

    Ok(())
}

