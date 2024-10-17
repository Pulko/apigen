use crate::{builder, template::TemplateConfig};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Failed generating API folder. {0}")]
    APIFolderError(#[from] crate::builder::BuilderError),
    #[error("Schema must contain at least one entity")]
    EmptySchemaError,
    #[error("Entity name cannot be empty")]
    EntityNameError,
    #[error("Error parsing schema")]
    ParsingError(#[from] serde_json::Error),
    #[error("Entity {0} must contain at least one field.")]
    EmptyEntityError(String),
    #[error("Field name cannot be empty in entity '{0}'.")]
    FieldNameError(String),
    #[error("Field type cannot be empty for field '{0}' in entity '{1}'.")]
    FieldTypeError(String, String),
    #[error("First field name must be 'id'")]
    EntityIdError,
    #[error("Field {0} must contain at least one field.")]
    EmptyFieldError(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiSchema {
    pub entities: Vec<Entity>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Entity {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Field {
    pub name: String,
    pub field_type: String,
}

pub struct Schema {
    pub json: ApiSchema,
}

impl Schema {
    pub fn new(json: serde_json::Value) -> Result<Self, SchemaError> {
        let api_schema;

        match Schema::parse_schema(json) {
            Ok(schema) => api_schema = schema,
            Err(e) => return Err(e),
        }

        return match Schema::validate_schema(&api_schema) {
            Ok(_) => Ok(Self { json: api_schema }),
            Err(e) => {
                return Err(e);
            }
        };
    }

    pub async fn generate(
        &self,
        project_id: &str,
        template_config: TemplateConfig,
    ) -> Result<String, SchemaError> {
        let api_schema = &self.json;

        let generation_result =
            builder::generate_api_folder(project_id, api_schema, &template_config).await;

        return match generation_result {
            Ok(result) => Ok(result),
            Err(e) => Err(SchemaError::APIFolderError(e).into()),
        };
    }

    fn parse_schema(json: serde_json::Value) -> Result<ApiSchema, SchemaError> {
        let api_schema: Result<ApiSchema, serde_json::Error> = serde_json::from_value(json);

        match api_schema {
            Ok(schema) => Ok(schema),
            Err(e) => Err(SchemaError::ParsingError(e)),
        }
    }

    fn validate_schema(api_schema: &ApiSchema) -> Result<(), SchemaError> {
        if api_schema.entities.is_empty() {
            return Err(SchemaError::EmptySchemaError.into());
        }

        if api_schema.entities[0].fields.is_empty() {
            return Err(SchemaError::EmptyFieldError(
                api_schema.entities[0].name.clone(),
            ));
        }

        if api_schema.entities[0].fields[0].name.trim() != "id" {
            return Err(SchemaError::EntityIdError.into());
        }

        for entity in &api_schema.entities {
            if entity.name.trim().is_empty() {
                return Err(SchemaError::EntityNameError.into());
            }
            if entity.fields.is_empty() {
                return Err(SchemaError::EmptyEntityError(entity.name.clone()));
            }
            for field in &entity.fields {
                if field.name.trim().is_empty() {
                    return Err(SchemaError::FieldNameError(entity.name.clone()).into());
                }
                if field.field_type.trim().is_empty() {
                    return Err(SchemaError::FieldTypeError(
                        field.name.clone(),
                        entity.name.clone(),
                    ));
                }
            }
        }

        Ok(())
    }
}
