use super::api_schema::ApiSchema;
use crate::template::TemplateConfig;
use std::collections::HashMap;
use tera::{Context, Result as TeraResult, Tera, Value};
use thiserror::Error;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Error adding embedded templates")]
    AddingRawTemplateError,
    #[error("Error creating folder")]
    CreatingFolderError,
    #[error("Error reading template file")]
    ReadingTemplateError,
}

async fn add_templates_from_config<'a>(
    tera: &'a mut Tera,
    config: &'a TemplateConfig,
) -> Result<&'a mut Tera, BuilderError> {
    for (template_name, template_path) in &config.template_paths {
        match fs::read_to_string(template_path).await {
            Ok(content) => {
                if let Err(e) = tera.add_raw_template(template_name, &content) {
                    eprintln!("Error adding template '{}': {}", template_name, e);
                    return Err(BuilderError::AddingRawTemplateError);
                }
            }
            Err(e) => {
                eprintln!("Error reading template file '{}': {}", template_path, e);
                return Err(BuilderError::ReadingTemplateError);
            }
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
        Err(e) => {
            eprintln!("Failed adding templates from config: {}", e);
            return Err(BuilderError::AddingRawTemplateError.into());
        }
    };

    tera.register_filter("capitalize_first_letter", capitalize_filter);
    tera.register_filter("diesel_type", diesel_type_filter);
    tera.register_filter("pluralize", pluralize_filter);

    let folder = generate_folder_name(project_id);

    if let Err(e) = fs::create_dir_all(format!("output/{}/src/api", folder)).await {
        eprintln!("Error creating folder: {}", e);
        return Err(BuilderError::CreatingFolderError.into());
    }

    if let Err(e) = render(tera, &context, &folder, api_schema, config).await {
        eprintln!("Error rendering templates: {}", e);
        return Err(BuilderError::AddingRawTemplateError);
    }

    Ok(folder)
}

async fn render(
    tera: &mut Tera,
    context: &Context,
    folder: &str,
    api_schema: &ApiSchema,
    config: &TemplateConfig,
) -> Result<(), std::io::Error> {
    render_single_template(
        tera,
        context,
        "schema.rs",
        &format!("output/{}/src/schema.rs", folder),
    )
    .await?;

    render_single_template(
        tera,
        context,
        "main.rs",
        &format!("output/{}/src/main.rs", folder),
    )
    .await?;

    for entity in &api_schema.entities {
        let mut entity_context = Context::new();
        entity_context.insert("entity", entity);

        render_single_template(
            tera,
            &entity_context,
            "entity.rs",
            &format!("output/{}/src/api/{}.rs", folder, entity.name),
        )
        .await?;
    }

    render_single_template(
        tera,
        context,
        "mod.rs",
        &format!("output/{}/src/api/mod.rs", folder),
    )
    .await?;

    copy_additional_file("Cargo.toml", folder, config).await?;
    copy_additional_file(".gitignore", folder, config).await?;
    copy_additional_file("Dockerfile", folder, config).await?;

    Ok(())
}

async fn render_single_template(
    tera: &mut Tera,
    context: &Context,
    template_name: &str,
    output_file_path: &str,
) -> Result<(), std::io::Error> {
    if let Ok(content) = tera.render(template_name, context) {
        let mut output_file = File::create(output_file_path).await?;
        output_file.write_all(content.as_bytes()).await?;
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Template not found: {}", template_name),
        ));
    }

    Ok(())
}

async fn copy_additional_file(
    file_key: &str,
    folder: &str,
    config: &TemplateConfig,
) -> Result<(), std::io::Error> {
    if let Some(template_path) = config.get_template_path(file_key) {
        let template_content = fs::read_to_string(template_path).await?;

        let output_file_path = match file_key {
            "Cargo.toml" => format!("output/{}/Cargo.toml", folder),
            ".gitignore" => format!("output/{}/.gitignore", folder),
            "Dockerfile" => format!("output/{}/Dockerfile", folder),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Unknown file key: {}", file_key),
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
