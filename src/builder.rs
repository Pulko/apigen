use std::collections::HashMap;

use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use tera::{Context, Result as TeraResult, Tera, Value};

use crate::template::TemplateConfig;

use super::api_schema::ApiSchema;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Error adding embedded templates")]
    AddingRawTemplateError,
    #[error("Error creating folder")]
    CreatingFolderError,
    #[error("Error reading template file")]
    ReadingTemplateError,
}

pub async fn add_templates_from_config<'a>(
    tera: &'a mut Tera,
    config: &'a TemplateConfig,
) -> Result<&'a mut Tera, BuilderError> {
    for (template_name, template_path) in &config.template_paths {
        if let Ok(content) = fs::read_to_string(template_path).await {
            let _ = tera.add_raw_template(template_name, &content);
        } else {
            return Err(BuilderError::ReadingTemplateError.into());
        }
    }
    Ok(tera)
}

pub async fn generate_api_folder(
    project_id: &str,
    api_schema: &ApiSchema,
    config: &TemplateConfig,
) -> Result<String, BuilderError> {
    let mut context = Context::new();
    context.insert("entities", &api_schema.entities);

    let mut original_tera = Tera::default();
    let tera = add_templates_from_config(&mut original_tera, config).await;

    let tera = match tera {
        Ok(t) => t,
        Err(_) => return Err(BuilderError::AddingRawTemplateError.into()),
    };

    tera.register_filter("capitalize_first_letter", capitalize_filter);
    tera.register_filter("diesel_type", diesel_type_filter);
    tera.register_filter("pluralize", pluralize_filter);

    let folder = generate_folder_name(project_id);

    if let Err(_e) = fs::create_dir_all(format!("output/{}/src/api", folder)).await {
        return Err(BuilderError::CreatingFolderError.into());
    }

    let _ = render_template(tera, &context, &folder, api_schema).await;

    Ok(folder)
}

async fn render_template(tera: &mut Tera, context: &Context, folder: &str, api_schema: &ApiSchema) {
    // Render schema.rs
    if let Ok(schema_ts) = tera.render("schema.rs", &context) {
        let mut schema_file = File::create(format!("output/{}/src/schema.rs", folder))
            .await
            .unwrap();
        schema_file.write_all(schema_ts.as_bytes()).await.unwrap();
    }

    // Render main.rs
    if let Ok(main_rs) = tera.render("main.rs", &context) {
        let mut main_file = File::create(format!("output/{}/src/main.rs", folder))
            .await
            .unwrap();
        main_file.write_all(main_rs.as_bytes()).await.unwrap();
    }

    // Render entity files inside api/
    for entity in &api_schema.entities {
        let mut entity_context = Context::new();
        entity_context.insert("entity", entity);

        if let Ok(entity_rs) = tera.render("entity.rs", &entity_context) {
            let mut entity_file =
                File::create(format!("output/{}/src/api/{}.rs", folder, entity.name))
                    .await
                    .unwrap();
            entity_file.write_all(entity_rs.as_bytes()).await.unwrap();
        }
    }

    // Render api/mod.rs
    if let Ok(mod_rs) = tera.render("mod.rs", &context) {
        let mut mod_file = File::create(format!("output/{}/src/api/mod.rs", folder))
            .await
            .unwrap();
        mod_file.write_all(mod_rs.as_bytes()).await.unwrap();
    }

    // Copy additional files like Cargo.toml, .gitignore, and Dockerfile
    copy_additional_file("Cargo.toml", folder).await.unwrap();
    copy_additional_file(".gitignore", folder).await.unwrap();
    copy_additional_file("Dockerfile", folder).await.unwrap();
}

async fn copy_additional_file(file_key: &str, folder: &str) -> Result<(), std::io::Error> {
    let config = TemplateConfig::new("postgres", "axum"); // This should ideally be passed as an argument, hardcoded for now

    if let Some(template_path) = config.get_template_path(file_key) {
        let template_content = fs::read_to_string(template_path).await?;

        let output_file_path = match file_key {
            "Cargo.toml" => format!("output/{}/Cargo.toml", folder),
            ".gitignore" => format!("output/{}/.gitignore", folder),
            "Dockerfile" => format!("output/{}/Dockerfile", folder),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Unknown file key",
                ))
            }
        };

        let mut output_file = File::create(output_file_path).await?;
        output_file.write_all(template_content.as_bytes()).await?;
    }

    Ok(())
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
