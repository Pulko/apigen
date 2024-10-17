use api_schema::Schema;
use clap::{Arg, Command};
use serde_json::Value;
use short_uuid::ShortUuid;
use template::TemplateConfig;

mod api_schema;
mod builder;
mod template;

const SUPPORTED_DBS: [&str; 1] = ["postgres"];
const SUPPORTED_FRAMEWORKS: [&str; 1] = ["axum"];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("RAPIDO: API Generator")
        .version("0.1.0")
        .author("Fedor Tkachenko <vzdbovich@gmail.com>")
        .about("Generates APIs based on a provided schema")
        .arg(
            Arg::new("api_schema")
                .help("The JSON schema for the API")
                .required(true),
        )
        .arg(
            Arg::new("db_type")
                .help("The database type (supported: postgres)")
                .default_value("postgres"),
        )
        .arg(
            Arg::new("framework")
                .help("The framework to use (supported: axum)")
                .default_value("axum"),
        )
        .get_matches();

    let api_schema_json = matches.get_one::<String>("api_schema").unwrap();
    let db_type = matches.get_one::<String>("db_type").unwrap().to_lowercase();
    let framework = matches
        .get_one::<String>("framework")
        .unwrap()
        .to_lowercase();

    let api_schema_value: Value = match serde_json::from_str(api_schema_json) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Invalid JSON schema provided: {}", e);
            std::process::exit(1);
        }
    };

    let template_config = TemplateConfig::new(&db_type, &framework);

    if !is_valid_config(&template_config) {
        eprintln!(
            "Unsupported configuration.\n\nSupported db types: {}.\nSupported frameworks: {}.",
            SUPPORTED_DBS.join(", "),
            SUPPORTED_FRAMEWORKS.join(", ")
        );
        std::process::exit(1);
    }

    let project_id = generate_short_hash();

    let schema = match Schema::new(api_schema_value) {
        Ok(schema) => schema,
        Err(e) => {
            eprintln!("Error with schema: {}", e);
            std::process::exit(1);
        }
    };

    match schema.generate(&project_id, template_config).await {
        Ok(output) => {
            println!("API generated successfully: {}", output);
        }
        Err(e) => eprintln!("Error generating API: {}", e),
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
