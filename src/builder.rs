use super::api_schema::ApiSchema;
use crate::template::TemplateConfig;
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use tera::{Context, Result as TeraResult, Tera, Value};
use thiserror::Error;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

const TEMPLATES_DIR: Dir = include_dir!("src/templates");

#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Error adding embedded templates. {0}")]
    AddingRawTemplateError(String),
    #[error("Error creating all directories tree. {0}")]
    CreatingFolderError(String),
    #[error("Error reading template file")]
    ReadingTemplateError(String),
    #[error("Error rendering single template: {0}")]
    RenderingTemplateError(String),
}

async fn add_templates_from_config<'a>(
    tera: &'a mut Tera,
    config: &'a TemplateConfig,
) -> Result<&'a mut Tera, BuilderError> {
    let current_dir = TEMPLATES_DIR.get_dir(config.db.as_str()).ok_or_else(|| {
        BuilderError::ReadingTemplateError(format!("Directory '{}' not found", config.db))
    })?;

    for (template_name, template_path) in &config.template_paths {
        let file = current_dir.get_file(template_path).ok_or_else(|| {
            return BuilderError::ReadingTemplateError(format!(
                "Template '{}' not found",
                template_path
            ));
        })?;

        let content = file.contents_utf8().ok_or_else(|| {
            return BuilderError::ReadingTemplateError(format!(
                "Template '{}' is not valid UTF-8",
                template_path
            ));
        })?;

        tera.add_raw_template(template_name, content).map_err(|e| {
            return BuilderError::AddingRawTemplateError(format!(
                "Error adding template '{}': {}",
                template_name, e
            ));
        })?;
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
    let tera = add_templates_from_config(&mut original_tera, config)
        .await
        .map_err(|e| BuilderError::AddingRawTemplateError(format!("{:?}", e)))?;

    tera.register_filter("capitalize_first_letter", capitalize_filter);
    tera.register_filter("diesel_type", diesel_type_filter);
    tera.register_filter("pluralize", pluralize_filter);

    let folder = generate_folder_name(project_id);

    fs::create_dir_all(format!("{}/src/api", folder))
        .await
        .map_err(|e| BuilderError::CreatingFolderError(format!("{:?}", e)))?;

    render(tera, &context, &folder, api_schema)
        .await
        .map_err(|e| BuilderError::AddingRawTemplateError(format!("{:?}", e)))?;

    Ok(folder)
}

async fn render(
    tera: &mut Tera,
    context: &Context,
    folder: &str,
    api_schema: &ApiSchema,
) -> Result<(), BuilderError> {
    render_single_template(tera, context, "main.rs", &format!("{}/src/main.rs", folder)).await?;

    for entity in &api_schema.entities {
        let mut entity_context = Context::new();
        entity_context.insert("entity", entity);

        render_single_template(
            tera,
            &entity_context,
            "entity.rs",
            &format!("{}/src/api/{}.rs", folder, entity.name.to_lowercase()),
        )
        .await?;
    }

    render_single_template(
        tera,
        context,
        "mod.rs",
        &format!("{}/src/api/mod.rs", folder),
    )
    .await?;

    render_single_template(
        tera,
        context,
        "Cargo.toml",
        &format!("{}/Cargo.toml", folder),
    )
    .await?;

    render_single_template(
        tera,
        context,
        ".gitignore",
        &format!("{}/.gitignore", folder),
    )
    .await?;

    render_single_template(tera, context, ".env", &format!("{}/.env", folder)).await?;

    render_single_template(
        tera,
        context,
        "docker-compose.yml",
        &format!("{}/docker-compose.yml", folder),
    )
    .await?;

    Ok(())
}

async fn render_single_template(
    tera: &mut Tera,
    context: &Context,
    template_name: &str,
    output_file_path: &str,
) -> Result<(), BuilderError> {
    let content = tera
        .render(template_name, context)
        .map_err(|_| BuilderError::RenderingTemplateError(format!("{}", template_name)))?;

    let mut output_file = File::create(output_file_path)
        .await
        .map_err(|_| BuilderError::RenderingTemplateError(format!("{}", template_name)))?;

    output_file
        .write_all(content.as_bytes())
        .await
        .map_err(|_| BuilderError::RenderingTemplateError(format!("{}", template_name)))?;

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
        "i32" => "Integer",
        "u32" => "Integer",
        "String" => "Text",
        "Option<String>" => "Nullable<Text>",
        "Option<u32>" => "Nullable<Integer>",
        "Vec<String>" => "Array<Text>",
        "Vec<u32>" => "Array<Integer>",
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
