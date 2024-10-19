use std::collections::HashMap;

#[derive(Debug)]
pub struct TemplateConfig {
    pub db: String,                              // e.g., "postgres"
    pub framework: String,                       // e.g., "axum"
    pub template_paths: HashMap<String, String>, // template file paths
}

const POSTGRES_TEMPLATES: [(&str, &str); 7] = [
    ("schema.rs", "postgres/schema.rs.tera"),
    ("main.rs", "postgres/main.rs.tera"),
    ("entity.rs", "postgres/api/entity.rs.tera"),
    ("mod.rs", "postgres/api/mod.rs.tera"),
    ("Cargo.toml", "postgres/Cargo.toml.template"),
    (".gitignore", "postgres/.gitignore.template"),
    ("Dockerfile", "postgres/Dockerfile.template"),
];

const SUPPORTED_DBS: [&str; 1] = ["postgres"];
const SUPPORTED_FRAMEWORKS: [&str; 1] = ["axum"];

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

        match db_name {
            "postgres" => {
                for (_, template) in POSTGRES_TEMPLATES.iter().enumerate() {
                    template_paths.insert(template.0.to_string(), template.1.to_string());
                }
            }
            _ => {}
        }

        Self {
            db: db_name.to_string(),
            framework: framework_name.to_string(),
            template_paths,
        }
    }

    pub fn is_valid(&self) -> bool {
        SUPPORTED_DBS.contains(&self.db.as_str())
            && SUPPORTED_FRAMEWORKS.contains(&self.framework.as_str())
    }

    pub fn get_supported_config_message(&self) -> String {
        format!(
            "Supported configurations:\n\n\ndatabases:\n {:?},\n\nframework:\n{:?}\n\n\n\nGot invalid configuration: db: {}, framework: {}",
            SUPPORTED_DBS.join(",\n"),
            SUPPORTED_FRAMEWORKS.join(",\n"),
            self.db,
            self.framework
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_config_postgres() {
        let config = TemplateConfig::new("postgres", "axum");
        assert_eq!(config.db, "postgres");
        assert_eq!(config.framework, "axum");
        assert!(config.is_valid());
    }

    #[test]
    fn test_template_config_invalid() {
        let config = TemplateConfig::new("unknown_db", "axum");
        assert!(!config.is_valid());
    }
}
