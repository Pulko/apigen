use api_schema::Schema;
use clap::{Arg, Command};
use serde_json::Value;
use short_uuid::ShortUuid;
use template::TemplateConfig;

mod api_schema;
mod builder;
mod template;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("apigen: API Generator")
        .version("0.2.0")
        .author("Fedor Tkachenko <vzdbovich@gmail.com>")
        .about("Generates APIs based on a provided schema")
        .arg(
            Arg::new("api_schema")
                .help("The JSON schema for the API")
                .required(true),
        )
        .arg(
            Arg::new("db_type")
                .help("The database type (default: postgres)")
                .default_value("postgres"),
        )
        .get_matches();

    let api_schema_json = matches.get_one::<String>("api_schema").unwrap();
    let db_type = matches.get_one::<String>("db_type").unwrap().to_lowercase();

    let api_schema_value: Value = match serde_json::from_str(api_schema_json) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Invalid JSON schema provided: {}", e);
            std::process::exit(1);
        }
    };

    let template_config = TemplateConfig::new(&db_type);

    if !template_config.is_valid() {
        eprintln!("{}", template_config.get_supported_config_message());
        std::process::exit(1);
    }

    let project_id = generate_short_hash();

    let schema = match Schema::new(api_schema_value) {
        Ok(schema) => schema,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let entities_to_print = schema
        .json
        .entities
        .iter()
        .map(|e| e.name.clone())
        .collect::<Vec<String>>();

    match schema.generate(&project_id, template_config).await {
        Ok(output) => {
            println!("✅ API generation completed successfully: {} \n\n", output);
            println!("1️⃣  Start the database container with Docker Compose:");
            println!("      ⎯ docker compose up --build\n");
            println!("2️⃣  Set up the Diesel CLI if not already installed. You can install it with:");
            println!(
                "      ⎯ cargo install diesel_cli --no-default-features --features postgres\n"
            );
            println!("3️⃣  Configure your database connection:");
            println!("      Ensure `DATABASE_URL` is correctly set in your environment, e.g., in a `.env` file.");
            println!("      Then, initialize Diesel with the following commands:\n");
            println!("      ⎯ diesel setup                    # Set up the database\n");
            println!(
                "4️⃣  Now you can run the following commands to add your entities to the database:"
            );
            for entity in entities_to_print {
                println!(
                    "      ⎯ diesel migration generate {}            # Apply the migration for the {} table\n",
                    entity.to_ascii_lowercase() + "s", entity
                );
            }
            // color the output ⎯
            println!("      ⎯ diesel migration run            # Apply all pending migrations\n");
            println!("📘 Refer to the Diesel documentation for more details: https://diesel.rs/guides/getting-started/");
            println!("\nHappy coding! 🎉");
        }
        Err(e) => eprintln!("{}", e),
    }

    Ok(())
}

fn generate_short_hash() -> String {
    ShortUuid::generate().to_string()
}
