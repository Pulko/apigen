use std::collections::HashMap;

use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use tera::{Context, Result as TeraResult, Tera, Value};

use super::api_schema::ApiSchema;

pub async fn generate_api_folder(
    project_id: &str,
    api_schema: &ApiSchema,
) -> Result<String, std::io::Error> {
    let mut context = Context::new();
    context.insert("entities", &api_schema.entities);
    let mut tera = match Tera::new("src/templates/postgres/**/*.tera") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };

    tera.register_filter("capitalize_first_letter", capitalize_filter);
    tera.register_filter("diesel_type", diesel_type_filter);
    tera.register_filter("pluralize", pluralize_filter);

    let folder = generate_folder_name(project_id);

    fs::create_dir_all(format!("output/{}/src/api", folder))
        .await
        .unwrap();

    println!("{:?}", tera.get_template_names().collect::<Vec<&str>>());

    // Render schema.rs
    let schema_ts = tera.render("schema.rs.tera", &context).unwrap();
    let mut schema_file = File::create(format!("output/{}/src/schema.rs", folder))
        .await
        .unwrap();
    schema_file.write_all(schema_ts.as_bytes()).await.unwrap();

    // Render main.rs
    let main_rs = tera.render("main.rs.tera", &context).unwrap();
    let mut main_file = File::create(format!("output/{}/src/main.rs", folder))
        .await
        .unwrap();
    main_file.write_all(main_rs.as_bytes()).await.unwrap();

    // Render entity files inside api/
    for entity in &api_schema.entities {
        let mut entity_context = Context::new();
        entity_context.insert("entity", entity);

        let entity_rs = tera.render("api/entity.rs.tera", &entity_context).unwrap();
        let mut entity_file = File::create(format!("output/{}/src/api/{}.rs", folder, entity.name))
            .await
            .unwrap();
        entity_file.write_all(entity_rs.as_bytes()).await.unwrap();
    }

    // Render api/mod.rs
    let mod_rs = tera.render("api/mod.rs.tera", &context).unwrap();
    let mut mod_file = File::create(format!("output/{}/src/api/mod.rs", folder))
        .await
        .unwrap();
    mod_file.write_all(mod_rs.as_bytes()).await.unwrap();

    // Copy cargo.toml.template content
    let cargo_toml_template = fs::read_to_string("src/templates/postgres/cargo.toml.template")
        .await
        .unwrap();
    let mut cargo_file = File::create(format!("output/{}/Cargo.toml", folder))
        .await
        .unwrap();
    cargo_file
        .write_all(cargo_toml_template.as_bytes())
        .await
        .unwrap();

    // Copy .gitignore.template content
    let gitignore_template = fs::read_to_string("src/templates/postgres/.gitignore.template")
        .await
        .unwrap();
    let mut gitignore_file = File::create(format!("output/{}/.gitignore", folder))
        .await
        .unwrap();
    gitignore_file
        .write_all(gitignore_template.as_bytes())
        .await
        .unwrap();

    // Copy Dockerfile.template content
    let dockerfile_template = fs::read_to_string("src/templates/postgres/Dockerfile.template")
        .await
        .unwrap();
    let mut dockerfile_file = File::create(format!("output/{}/Dockerfile", folder))
        .await
        .unwrap();
    dockerfile_file
        .write_all(dockerfile_template.as_bytes())
        .await
        .unwrap();

    Ok(folder)
}

pub fn generate_folder_name(project_id: &str) -> String {
    format!("project_{}", project_id)
}

fn capitalize_filter(value: &Value, _: &HashMap<String, Value>) -> TeraResult<Value> {
    if let Some(s) = value.as_str() {
        if let Some(c) = s.chars().next() {
            let capitalized = c.to_uppercase().to_string() + &s[1..];
            return Ok(tera::to_value(capitalized)?);
        }
    }
    Ok(value.clone())
}

fn diesel_type_filter(value: &Value, _: &HashMap<String, Value>) -> TeraResult<Value> {
    let rust_type = value.as_str().unwrap();
    let diesel_type = match rust_type {
        "u32" => "Int4",
        "String" => "Text",
        "Option<String>" => "Nullable<Text>",
        "Option<u32>" => "Nullable<Int4>",
        "Vec<String>" => "Array<Text>",
        "Vec<u32>" => "Array<Int4>",
        "Vec<Option<String>>" => "Array<Nullable<Text>>",
        "Value" => "Jsonb",
        _ => rust_type,
    };
    Ok(Value::String(diesel_type.to_string()))
}

fn pluralize_filter(value: &Value, _: &HashMap<String, Value>) -> TeraResult<Value> {
    let singular = value.as_str().unwrap();
    let plural = if singular.ends_with('s') {
        singular.to_string()
    } else {
        format!("{}s", singular)
    };
    Ok(Value::String(plural))
}
