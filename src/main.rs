#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::http::Method;
use rocket_contrib::json::Json;
use rocket_cors::AllowedHeaders;
use rocket_cors::AllowedOrigins;
use rocket_cors::CorsOptions;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Serialize)]
struct CompileResult {
    c_code: String,
    stdout: String,
    stderr: String,
    has_error: bool,
}

#[post("/vala_to_c", data = "<valacode>")]
fn vala_to_c(valacode: String) -> Json<CompileResult> {
    let compile_result = run_valac(valacode);
    return Json(compile_result);
}

fn main() {
    let allowed_origins =
        AllowedOrigins::some_exact(&["https://hoppscotch.io", "http://localhost:5000"]);

    // You can also deserialize this
    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::All,
        allow_credentials: true,
        ..Default::default()
    };

    rocket::ignite()
        .mount("/", routes![index, vala_to_c])
        .attach(cors.to_cors().expect("To not fail"))
        .launch();
}

fn run_valac(vala_code: String) -> CompileResult {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    println!("{:?}", since_the_epoch);

    let time = since_the_epoch.as_millis();
    let file_name_vala = String::from(time.to_string() + ".vala");

    let path = Path::new(&file_name_vala);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(vala_code.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }

    let mut command = Command::new("valac");
    command.arg("-C").arg(file_name_vala.clone());
    let output = command.output().expect("failed to execute process");

    let mut stdout_text = "".to_string();
    let mut stderr_text = "".to_string();

    match str::from_utf8(&output.stdout) {
        Ok(v) => {
            stdout_text = v.to_string();
            println!("{}", v);
        }
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    match str::from_utf8(&output.stderr) {
        Ok(v) => {
            stderr_text = v.to_string();
            println!("{}", v);
        }
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    let file_name_c = String::from(time.to_string() + ".c");

    let mut result = CompileResult {
        c_code: "".to_string(),
        stdout: stdout_text,
        stderr: stderr_text,
        has_error: true,
    };

    match fs::read_to_string(file_name_c.clone()) {
        Ok(c_code) => {
            result.c_code = c_code;
            result.has_error = false;
        }
        Err(e) => {
            println!("{}", e);
            result.has_error = true;
        }
    };

    remove_files(file_name_vala, file_name_c);
    return result;
}

fn remove_files(path1: String, path2: String) {
    delete_file(path1);
    delete_file(path2);
}

fn delete_file(path: String) {
    match fs::remove_file(path.clone()) {
        Ok(v) => println!("{} deleted", path.clone()),
        Err(e) => println!("{} could not deleted", path),
    };
}
