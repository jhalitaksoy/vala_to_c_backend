#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket_contrib::json::Json;
use std::str;

mod cors;
mod valac;

#[get("/")]
fn index() -> &'static str {
    "Server is running!"
}

#[post("/vala_to_c", data = "<valacode>")]
fn vala_to_c(valacode: String) -> Json<valac::CompileResult> {
    let compile_result = valac::compile(valacode);
    return Json(compile_result);
}

fn main() {
    let cors_options = cors::get_cors_options();
    rocket::ignite()
        .mount("/", routes![index, vala_to_c])
        .attach(cors_options.to_cors().expect("To not fail"))
        .launch();
}

