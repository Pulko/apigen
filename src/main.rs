use api_schema::Schema;
use serde_json::Value;
use short_uuid::ShortUuid;
use std::env;
use std::process;
use template::TemplateConfig;

mod api_schema;
mod builder;
mod template;

const USAGE: &str = "<api_schema> <?db_type> <?framework>";
const SUPPORTED_DBS: [&str; 1] = ["postgres"];
const SUPPORTED_FRAMEWORKS: [&str; 1] = ["axum"];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let len = args.len();

    if len < 1 {
        eprintln!("Missing arguments. Usage: {} {}", args[0], USAGE);
        process::exit(1);
    }

    let api_schema_json = &args[1];
    let db = if len > 2 { &args[2].to_lowercase() } else { "" };
    let framework = if len > 3 { &args[3].to_lowercase() } else { "" };

    let api_schema_value: Value = match serde_json::from_str(api_schema_json) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Invalid JSON schema provided: {}", e);
            process::exit(1);
        }
    };

    let template_config = TemplateConfig::new(&db.to_lowercase(), &framework.to_lowercase());

    if !is_valid_config(&template_config) {
        eprintln!(
            "Unsupported configuration.\n\nSupported db types: {}.\n\nSupported frameworks: {}.",
            SUPPORTED_DBS.join(", "),
            SUPPORTED_FRAMEWORKS.join(", ")
        );
        process::exit(1);
    }

    let project_id = generate_short_hash();

    let schema = Schema::new(api_schema_value);

    if schema.is_err() {
        eprintln!("Error: {}", schema.err().unwrap());
        process::exit(1);
    }

    match schema?.generate(&project_id, template_config).await {
        Ok(output) => {
            println!("API generated successfully: {}", output);
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}

fn generate_short_hash() -> String {
    ShortUuid::generate().to_string()
}

fn is_valid_config(template_config: &TemplateConfig) -> bool {
    SUPPORTED_DBS.contains(&template_config.db.as_str())
        && SUPPORTED_FRAMEWORKS.contains(&template_config.framework.as_str())
}
