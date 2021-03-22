#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::http::Method;
use rocket_cors::AllowedHeaders;
use rocket_cors::AllowedOrigins;
use rocket_cors::CorsOptions;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Responder)]
enum MyResult {
    #[response(status = 200)]
    Ok(String),
    #[response(status = 409)]
    Err(String),
}

#[derive(Serialize)]
struct CompilerOutput{
    c_code : String,
    stdout : String,
    stderr : String,
    has_error : bool,
}

#[post("/vala_to_c", data = "<valacode>")]
fn vala_to_c(valacode: String) -> MyResult {
    let result = run_valac(valacode);
    if result.has_error {
        println!("Error **** {}", result.error);
        return MyResult::Err(result.error.to_string());
    } else {
        return MyResult::Ok(result.value.to_string());
    }
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

struct OperationResult {
    value: String,
    error: String,
    has_error: bool,
}

fn run_valac(vala_code: String) -> OperationResult {
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

    let mut result = OperationResult {
        value: "".to_string(),
        error: "".to_string(),
        has_error: false,
    };

    match fs::read_to_string(file_name_c.clone()) {
        Ok(c_code) => {
            result.value = c_code;
            result.has_error = false;
        }
        Err(e) => {
            println!("{}", e);
            result.error = format!("{} \n {}", stdout_text, stderr_text);
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
