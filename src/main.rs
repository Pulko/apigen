use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use tera::{Context, Filter, Result, Tera, Value};

mod schema;
use schema::ApiSchema;

#[get("/")]
async fn index() -> impl Responder {
    "Hello, world!"
}

struct CapitalizeFirstLetter;

impl Filter for CapitalizeFirstLetter {
    fn filter(&self, value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
        if let Some(s) = value.as_str() {
            if let Some(c) = s.chars().next() {
                let capitalized = c.to_uppercase().to_string() + &s[1..];
                return Ok(tera::to_value(capitalized)?);
            }
        }
        Ok(value.clone())
    }
}

#[post("/generate")]
async fn generate(api_schema: web::Json<ApiSchema>) -> impl Responder {
    let mut tera = match Tera::new("templates/**/*.rs.tera") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    tera.register_filter("capitalize_first_letter", CapitalizeFirstLetter);

    let mut context = Context::new();
    context.insert("entities", &api_schema.entities);

    // Ensure output directory and api sub-directory exist
    fs::create_dir_all("output/src/api").unwrap();

    // Render main.rs
    let main_rs = tera.render("main.rs.tera", &context).unwrap();
    let mut main_file = File::create("output/src/main.rs").unwrap();
    main_file.write_all(main_rs.as_bytes()).unwrap();

    // Render api/mod.rs
    let api_mod_rs = tera.render("api/mod.rs.tera", &context).unwrap();
    let mut api_mod_file = File::create("output/src/api/mod.rs").unwrap();
    api_mod_file.write_all(api_mod_rs.as_bytes()).unwrap();

    // Render entity files inside api/
    for entity in &api_schema.entities {
        context.insert("name", &entity.name);
        context.insert("fields", &entity.fields);
        let entity_rs = tera.render("api/entity.rs.tera", &context).unwrap();
        let mut entity_file = File::create(format!("output/src/api/{}.rs", entity.name)).unwrap();
        entity_file.write_all(entity_rs.as_bytes()).unwrap();
    }

    // Copy cargo.toml.template content
    let cargo_toml_template = fs::read_to_string("templates/cargo.toml.template").unwrap();
    let mut cargo_file = File::create("output/Cargo.toml").unwrap();
    cargo_file
        .write_all(cargo_toml_template.as_bytes())
        .unwrap();

    // Copy .gitignore.template content
    let gitignore_template = fs::read_to_string("templates/.gitignore.template").unwrap();
    let mut gitignore_file = File::create("output/.gitignore").unwrap();
    gitignore_file
        .write_all(gitignore_template.as_bytes())
        .unwrap();

    // zip output directory
    // remove output directory
    // and return the zip file and remove it after sending it

    let output = std::process::Command::new("zip")
        .arg("-r")
        .arg("output.zip")
        .arg("output")
        .output()
        .expect("failed to execute process");

    std::fs::remove_dir_all("output").unwrap();

    // create docker image out of the output folder

    HttpResponse::Ok().message_body("Generated successfully")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(generate))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
