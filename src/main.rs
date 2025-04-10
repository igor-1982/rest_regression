use clap::{Arg, ArgMatches, Command};
use anyhow;
use regex::Regex;
//use core::num;
use std::{collections::HashMap, env, fs::{self, File}, io::{BufRead, Read, Write}, path::Path};
use std::process::Command as ProcessCommand;
use walkdir::WalkDir;
use chrono::{DateTime, Local};

mod parse_grad;

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
        //.arg(Arg::new("task_status")
        //     .short('s')
        //     .long("task_status")
        //     .help("The status of the task to be tested")
        //)
        .get_matches()
}

pub fn is_file_exist_in_dir(dir_name: &Path, file_name: &str) -> anyhow::Result<bool> {
    let file_iter = fs::read_dir(dir_name)?;
    let mut is_exist = false;
    file_iter.into_iter().try_for_each(|x| {
        let entry = x.unwrap();
        let path = entry.path();
        if path.is_file() && path.ends_with(file_name) {
            is_exist = true;
            None
        } else {
            Some(())
        }
    });
    Ok(is_exist)
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

    let re_grad = Regex::new(
        r"[-]{3,} *Output gradient *\[[a-zA-Z.]*\] *[-]{3,}\n+(?P<grad>[\S\s]*)\n+[-]{3,}"
    ).unwrap();

    if re_grad.is_match(&buf) {
        let caps = re_grad.captures(&buf).unwrap();
        let grad = crate::parse_grad::parse_grad(&caps["grad"].to_string());
        result.insert("gradient".to_string(), grad);
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

fn main() -> anyhow::Result<()> {

    let s_time = init_timing();

    let mut is_pass = true;

    //let current_directory = env::current_dir().unwrap().as_os_str().to_os_string().to_str().unwrap().to_string();
    let current_directory = env::current_dir().unwrap().to_str().unwrap().to_string();
    let curr_path = Path::new(&current_directory);
    

    let rest_home = match env::var_os("REST_HOME") {
        Some(val) => val,
        _ => panic!("REST_HOME not defined in the environment."),
    };

    let input_folder = parse_input().get_one::<String>("regression_directory")
        .unwrap_or({
            let mut default_path = rest_home.clone();
            default_path.push("/rest_regression/bench_pool");
            &default_path.to_str().unwrap().to_string()
        }
        ).to_string();
    
    let work_folder = parse_input().get_one::<String>("working_directory").unwrap().to_string();
    let work_path = Path::new(&work_folder);

    let num_mpi = parse_input().get_one::<String>("n_mpi").unwrap().parse::<usize>().unwrap();
    println!("num_mpi = {}", num_mpi);


    fs::create_dir(work_path).unwrap_or(());
    //fs::create_dir(work_path).unwrap_err();

    let rest_mode = parse_input().get_one::<String>("rest_mode").unwrap().to_string();
    let rest_default_cmd = format!("{}/target/{}/rest", rest_home.to_str().unwrap(), &rest_mode);

    let rest_cmd = parse_input().get_one::<String>("rest_path").unwrap_or(&rest_default_cmd).to_string();
    //} else {
    //    let rest_cmd_pure = parse_input().get_one::<String>("rest_path").unwrap_or(&rest_default_cmd).to_string();
    //    format!("mpirun -n {} {}", &num_mpi, &rest_cmd_pure)
    //};
    if ! std::path::Path::new(&rest_cmd).is_file() {
        panic!("The REST binary \'{}\' does not exist", &rest_cmd);
    }

    println!("REST: {}", &rest_cmd);



    let mut job_index = 0;
    let mut fail_list = vec![];
    
    for entry in WalkDir::new(input_folder).into_iter().filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir() 
            && is_file_exist_in_dir(e.path(), &"reference.log").unwrap()
            //&& is_file_exist_in_dir(e.path(), &"ctrl.in").unwrap()
        ) {


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


        // enter the job folder
        env::set_current_dir(entry.path())?;
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

        let mut output_string = String::new();
        //println!("debug job_list {:?}", &job_list);
        for job in job_list {
            let output = if num_mpi == 1 {
                ProcessCommand::new(rest_cmd.as_str())
                    .arg("-i").arg(job).output().unwrap()
            } else {
                ProcessCommand::new("mpirun").arg("-n").arg(format!("{}",&num_mpi).as_str())
                    .arg(rest_cmd.as_str())
                    .arg("-i").arg(job).output().unwrap()
            };
            output_string = String::new();
            for line in output.stdout.lines() {
                output_string.push_str(line.unwrap().as_str());
                output_string.push_str("\n");
            }
        }
        // leave the job folder
        env::set_current_dir(curr_path)?;

        let mut ff = File::create(store_file.to_str().unwrap()).unwrap();
        ff.write(output_string.as_bytes())?;

        let out_hashmap = collect_results(store_file.to_str().unwrap())?;
        let is_pass_current = compare_results(&ref_hashmap, &out_hashmap);
        
        if ! is_pass_current {fail_list.push(entry.path().display().to_string());}

        is_pass = is_pass && is_pass_current;

        timing(&j_time, Some(&format!("Job {:3}", job_index)));
        job_index += 1;

    }

    if is_pass {
        println!("All regression tasks passed");
    } else {
        println!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        println!("Regression failed for");
        for job in fail_list {
            println!("X: \"{}\"", job);
        }
        println!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

    }

    timing(&s_time, Some(&"Total time consumption"));

    Ok(())
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

