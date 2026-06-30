use clap::{Arg, ArgMatches, Command};
use anyhow;
use regex::Regex;
//use core::num;
use std::{collections::HashMap, env, fs::{self, File}, io::{BufRead, Read, Write}, path::Path};
use std::process::Command as ProcessCommand;
use walkdir::WalkDir;
use chrono::{DateTime, Local};

mod parse_grad;

mod filter;

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

    let re_post_scf_eny = Regex::new(
      r"The \(R\)-xDH energy\s*:\s*(?P<energy>[\+-]?\d+.\d+)\s*"
    ).unwrap();

    if re_post_scf_eny.is_match(&buf) {
        let caps = re_post_scf_eny.captures(&buf).unwrap();
        let energy = caps.name("energy").unwrap().as_str().parse::<f64>().unwrap();
        result.insert("(R)-xDH energy".to_string(), vec![energy]);
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

    let re_hoco_luco_gap = Regex::new(
        r"HOCO-LUCO gap\s*:\s*(?P<energy>[-+]?\d+\.\d+)\s*eV"
        ).unwrap();

    if re_hoco_luco_gap.is_match(&buf) {
        let caps = re_hoco_luco_gap.captures(&buf).unwrap();
        let gap_energy = caps.name("energy").unwrap().as_str().parse::<f64>().unwrap();
        result.insert("hoco_luco_gap".to_string(),vec![gap_energy]);
    }

    // TDDFT excitation energies: extract first 3 excitation energies
    // Format from REST:
    // State       Energy (Ha)   Energy (eV)  Wavelength (nm)         Osc. Strength
    // --------------------------------------------------------------------------------
    //    1        0.11186693        3.0441
    //    2        0.12576545        3.4223
    //    3        0.13821993        3.7612
    let re_tddft = Regex::new(
        r"(?m)^\s*(?P<idx>\d+)\s+(?P<energy>[\+-]?\d+\.\d+)\s+(?P<ev>[\+-]?\d+\.\d+)\s+(?P<nm>[\+-]?\d+\.\d+)\s+(?P<osc>[\+-]?\d+\.\d+)"
    ).unwrap();

    let mut tddft_energies: Vec<f64> = vec![];
    let mut tddft_osc: Vec<f64> = vec![];
    for cap in re_tddft.captures_iter(&buf) {
        let idx = cap["idx"].parse::<usize>().unwrap_or(99);
        if idx <= 6 {
            tddft_energies.push(cap["energy"].parse::<f64>().unwrap());
            tddft_osc.push(cap["osc"].parse::<f64>().unwrap());
        }
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
        let ref_log = format!("{}/reference.log", entry.path().to_str().unwrap());
        let ref_hashmap = collect_results(&ref_log)?;
        

        // print "README" if exist
        if is_file_exist_in_dir(entry.path(), &"README").unwrap() {
            for line in fs::read_to_string(entry.path().join("README")).unwrap().lines() {
                println!("{}", line);
            }
        }


        // initialize the output file
        let mut store_file = work_path.as_os_str().to_os_string();
        store_file.push("/");
        store_file.push(entry.path()
              .file_name()
              .ok_or(format!("job_{}", job_index)).unwrap()
            );
        store_file.push(".log");
        println!("output file: {}", store_file.clone().into_string().unwrap());

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

        let job_work_dir = work_path.join(entry.path().file_name().unwrap());
        copy_dir_all(entry.path(), &job_work_dir)?;

        let test_name = entry.path().file_name().unwrap().to_str().unwrap();
        let (effective_num_mpi, effective_num_threads) = match test_config.get(test_name) {
            Some(config) if config.num_mpi.is_some() || config.num_threads.is_some() => {
                println!("Special test config: num_mpi={:?}, num_threads={:?}", config.num_mpi, config.num_threads);
                (config.num_mpi.unwrap_or(num_mpi), config.num_threads)
            },
            _ => (num_mpi, num_threads),
        };

        if let Some(nt) = effective_num_threads {
            for job_file_name in &job_list {
                let job_file_path = job_work_dir.join(job_file_name);
                if job_file_path.is_file() {
                    set_num_threads_in_file(&job_file_path, nt)?;
                    println!("Set num_threads = {} in {}", nt, job_file_name);
                }
            }
        }

        // enter the job folder
        env::set_current_dir(&job_work_dir)?;

        let mut output_string = String::new();
        for job in job_list {
            let output = if effective_num_mpi == 1 {
                ProcessCommand::new(rest_cmd.as_str())
                    .arg("-i").arg(job).output().unwrap()
            } else {
                ProcessCommand::new("mpirun").arg("-n").arg(format!("{}",&effective_num_mpi).as_str())
                    .arg(rest_cmd.as_str())
                    .arg("-i").arg(job).output().unwrap()
            };
            output_string = String::new();
            for line in output.stdout.lines() {
                output_string.push_str(line.unwrap().as_str());
                output_string.push_str("\n");
            }
            if !output.stderr.is_empty() {
                output_string.push_str("--- stderr ---\n");
                for line in output.stderr.lines() {
                    output_string.push_str(line.unwrap().as_str());
                    output_string.push_str("\n");
                }
            }
        }
        // Save Hessian/Frequencies output files before leaving & deleting the job dir
        for artifact_name in &["HessianMatrix.txt", "EigenModes.txt"] {
            let src = job_work_dir.join(artifact_name);
            if src.exists() {
                let mut dst_name = test_name.to_string();
                dst_name.push('_');
                dst_name.push_str(artifact_name);
                let _ = fs::copy(&src, work_path.join(&dst_name));
            }
        }

        // leave the job folder
        env::set_current_dir(curr_path)?;
        // remove the tmp job folder
        fs::remove_dir_all(&job_work_dir)?;

        let mut ff = File::create(store_file.to_str().unwrap()).unwrap();
        ff.write(output_string.as_bytes())?;

        let out_hashmap = collect_results(store_file.to_str().unwrap())?;
        let is_pass_current = compare_results(&ref_hashmap, &out_hashmap);
        
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

