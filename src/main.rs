use api_schema::{GeneratorError, Schema};
use serde_json::Value;
use short_uuid::ShortUuid;
use std::env;
use std::process;
use tokio::process::Command;

mod api_schema;
mod builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Missing argument - API Schema: Usage: {} <api_schema>",
            args[0]
        );
        process::exit(1);
    }

    let api_schema_json = &args[1];
    let api_schema_value: Value = serde_json::from_str(api_schema_json)?;

    let project_id = generate_short_hash();

    let schema = Schema::new(api_schema_value)?;

    match schema.generate(&project_id).await {
        Ok(tar_path) => {
            println!("API generated successfully: {}", tar_path);
            println!("Building Docker image...");

            if let Err(_e) = build_docker_image(&project_id).await {
                return Err(GeneratorError::DockerImageCreationError.into());
            } else {
                println!("Docker image built and pushed successfully.");
            }
        }
        Err(e) => eprintln!("Error generating API: {}", e),
    }

    Ok(())
}

async fn build_docker_image(project_id: &str) -> Result<(), GeneratorError> {
    let folder = format!("output/project_{}", project_id);

    let build_output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&folder)
        .output()
        .await
        .expect("Failed to build the project. Make sure Rust and Cargo are installed.");

    if !build_output.status.success() {
        return Err(GeneratorError::BuildError);
    }

    let dockerfile_path = format!("{}/Dockerfile", folder);
    let docker_image_name = format!("project_{}_image", project_id);

    let docker_build_output = Command::new("docker")
        .arg("build")
        .arg("-t")
        .arg(&docker_image_name)
        .arg("-f")
        .arg(&dockerfile_path)
        .arg(&folder)
        .output()
        .await
        .expect("Failed to build Docker image. Make sure Docker is installed.");

    if !docker_build_output.status.success() {
        return Err(GeneratorError::DockerImageCreationError);
    }

    println!("Docker image {} created successfully.", docker_image_name);

    let docker_push_output = Command::new("docker")
        .arg("push")
        .arg(&docker_image_name)
        .output()
        .await
        .expect("Failed to push Docker image. Make sure you are logged in to Docker.");

    if !docker_push_output.status.success() {
        return Err(GeneratorError::DockerImagePushError);
    }

    println!("Docker image {} pushed successfully.", docker_image_name);

    Ok(())
}

fn generate_short_hash() -> String {
    ShortUuid::generate().to_string()
}
