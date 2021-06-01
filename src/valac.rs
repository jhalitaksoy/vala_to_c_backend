use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
pub struct CompileResult {
    c_code: String,
    stdout: String,
    stderr: String,
    has_error: bool,
}

pub fn compile(vala_code: String) -> CompileResult {
    let time = get_time_as_milis();
    let file_name_vala = String::from(time.to_string() + ".vala");
    let file_name_c = String::from(time.to_string() + ".c");

    create_vala_file_and_write(&file_name_vala, &vala_code);
    let compile_result = run_valac_command(&file_name_vala, &file_name_c);
    remove_files(&file_name_vala, &file_name_c);

    return compile_result;
}

fn get_time_as_milis() -> u128 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let time = since_the_epoch.as_millis();
    return time;
}

fn create_vala_file_and_write(file_name: &String, vala_code: &String) {
    let path = Path::new(&file_name);
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };
    match file.write_all(vala_code.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

fn run_valac_command(file_name_vala : &String, file_name_c : &String) -> CompileResult{
    let mut command = Command::new("valac");
    command.arg("-C").arg(file_name_vala);
    //todo: check valac is installed
    let output = command.output().expect("failed to execute process");

    let mut compile_result = CompileResult {
        c_code:  "".to_string(),
        stdout: "".to_string(),
        stderr: "".to_string(),
        has_error: false,
    };
    match str::from_utf8(&output.stdout) {
        Ok(v) => {
            compile_result.stdout = v.to_string();
            println!("{}", v);
        }
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    match str::from_utf8(&output.stderr) {
        Ok(v) => {
            compile_result.stderr = v.to_string();
            println!("{}", v);
        }
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    match fs::read_to_string(file_name_c) {
        Ok(value) => {
            compile_result.c_code = value;
            compile_result.has_error = false;
        }
        Err(e) => {
            println!("{}", e);
            compile_result.has_error = true;
        }
    };
    return compile_result;
}

fn remove_files(path1: &String, path2: &String) {
    delete_file(path1);
    delete_file(path2);
}

fn delete_file(path: &String) {
    match fs::remove_file(&path) {
        Ok(_v) => println!("{} deleted", &path),
        Err(_e) => println!("{} could not deleted", &path),
    };
}
