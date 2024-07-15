use serde::{Deserialize, Serialize};
use tokio::process::Command;

use thiserror::Error;

use crate::builder::generate_api_folder;

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("Failed to create tar file")]
    TarCreationError,
    #[error("Failed to login to docker")]
    DockerLoginError,
    #[error("Failed to create docker image")]
    DockerImageCreationError,
    #[error("Failed to push docker image")]
    DockerImagePushError,
    #[error("Failed to remove docker image")]
    DockerImageRemoveError,
    #[error("Failed to build the project")]
    BuildError,
}

#[derive(Serialize, Deserialize)]
pub struct ApiSchema {
    pub entities: Vec<Entity>,
}

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: String,
}

pub struct Schema {
    pub json: ApiSchema,
}

impl Schema {
    pub fn new(json: serde_json::Value) -> Result<Self, serde_json::Error> {
        let json = serde_json::from_value(json);
        match json {
            Ok(json) => Ok(Self { json }),
            Err(e) => Err(e),
        }
    }

    pub async fn generate(&self, project_id: &str) -> Result<String, GeneratorError> {
        let api_schema = &self.json;

        let folder = generate_api_folder(project_id, api_schema).await.unwrap();

        // let build_output = Command::new("cargo")
        //     .arg("build")
        //     .current_dir(format!("output/{}", folder))
        //     .output()
        //     .await
        //     .unwrap();

        // if !build_output.status.success() {
        //     return Err(GeneratorError::BuildError.into());
        // }

        // let tar = Command::new("tar")
        //     .arg("-czvf")
        //     .arg(format!("output/{}.tar.gz", folder))
        //     .arg(format!("output/{}/target", folder))
        //     .output()
        //     .await
        //     .unwrap();

        // if !tar.status.success() {
        //     return Err(GeneratorError::TarCreationError.into());
        // }

        Ok(format!("output/{}.tar.gz", folder))
    }
}
