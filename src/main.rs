use api_schema::Schema;
use serde_json::Value;
use short_uuid::ShortUuid;
use std::env;
use std::process;
use template::TemplateConfig;

mod api_schema;
mod builder;
mod template;

const USAGE: &str = "<api_schema> <db_type> <framework>";
const SUPPORTED_DBS: [&str; 1] = ["postgres"];
const SUPPORTED_FRAMEWORKS: [&str; 1] = ["axum"];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Missing arguments. Usage: {} {}", args[0], USAGE);
        process::exit(1);
    }

    let api_schema_json = &args[1];
    let db_type = &args[2].to_lowercase();
    let framework = &args[3].to_lowercase();

    let api_schema_value: Value = match serde_json::from_str(api_schema_json) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Invalid JSON schema provided.");
            process::exit(1);
        }
    };

    let template_config = TemplateConfig::new(db_type, framework);

    if !is_valid_config(&template_config) {
        eprintln!(
            "Unsupported configuration.\n\nSupported db types: {}.\n\nSupported frameworks: {}.",
            SUPPORTED_DBS.join(", "),
            SUPPORTED_FRAMEWORKS.join(", ")
        );
        process::exit(1);
    }

    let project_id = generate_short_hash();

    let schema = Schema::new(api_schema_value)?;
    match schema.generate(&project_id, template_config).await {
        Ok(output) => {
            println!("API generated successfully: {}", output);
        }
        Err(e) => eprintln!("{}", e),
    }

    Ok(())
}

fn generate_short_hash() -> String {
    ShortUuid::generate().to_string()
}

fn is_valid_config(template_config: &TemplateConfig) -> bool {
    SUPPORTED_DBS.contains(&template_config.db_type.as_str())
        && SUPPORTED_FRAMEWORKS.contains(&template_config.framework.as_str())
}
