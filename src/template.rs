use std::collections::HashMap;

pub struct TemplateConfig {
    pub db_type: String,                         // e.g., "postgres"
    pub framework: String,                       // e.g., "axum"
    pub template_paths: HashMap<String, String>, // template file paths
}

impl TemplateConfig {
    pub fn new(db_type: &str, framework: &str) -> Self {
        let mut template_paths = HashMap::new();

        template_paths.insert(
            "schema.rs".into(),
            format!("templates/{}/schema.rs.tera", db_type),
        );
        template_paths.insert(
            "main.rs".into(),
            format!("templates/{}/main.rs.tera", db_type),
        );
        template_paths.insert(
            "entity.rs".into(),
            format!("templates/{}/api/entity.rs.tera", db_type),
        );
        template_paths.insert(
            "mod.rs".into(),
            format!("templates/{}/api/mod.rs.tera", db_type),
        );
        template_paths.insert(
            "Cargo.toml".into(),
            format!("templates/{}/Cargo.toml.template", db_type),
        );
        template_paths.insert(
            ".gitignore".into(),
            format!("templates/{}/.gitignore.template", db_type),
        );
        template_paths.insert(
            "Dockerfile".into(),
            format!("templates/{}/Dockerfile.template", db_type),
        );

        Self {
            db_type: db_type.to_string(),
            framework: framework.to_string(),
            template_paths,
        }
    }

    pub fn get_template_path(&self, key: &str) -> Option<&String> {
        self.template_paths.get(key)
    }
}
