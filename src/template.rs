use std::collections::HashMap;

#[derive(Debug)]
pub struct TemplateConfig {
    pub db: String,                              // e.g., "postgres"
    pub framework: String,                       // e.g., "axum"
    pub template_paths: HashMap<String, String>, // template file paths
}

impl TemplateConfig {
    pub fn new(db: &str, framework: &str) -> Self {
        let mut template_paths = HashMap::new();

        let db_name;
        let framework_name;

        if db.is_empty() {
            db_name = "postgres";
        } else {
            db_name = db;
        }

        if framework.is_empty() {
            framework_name = "axum";
        } else {
            framework_name = framework;
        }

        template_paths.insert(
            "schema.rs".into(),
            format!("src/templates/{}/schema.rs.tera", db_name),
        );
        template_paths.insert(
            "main.rs".into(),
            format!("src/templates/{}/main.rs.tera", db_name),
        );
        template_paths.insert(
            "entity.rs".into(),
            format!("src/templates/{}/api/entity.rs.tera", db_name),
        );
        template_paths.insert(
            "mod.rs".into(),
            format!("src/templates/{}/api/mod.rs.tera", db_name),
        );
        template_paths.insert(
            "Cargo.toml".into(),
            format!("src/templates/{}/Cargo.toml.template", db_name),
        );
        template_paths.insert(
            ".gitignore".into(),
            format!("src/templates/{}/.gitignore.template", db_name),
        );
        template_paths.insert(
            "Dockerfile".into(),
            format!("src/templates/{}/Dockerfile.template", db_name),
        );

        Self {
            db: db_name.to_string(),
            framework: framework_name.to_string(),
            template_paths,
        }
    }

    pub fn get_template_path(&self, key: &str) -> Option<&String> {
        self.template_paths.get(key)
    }
}
