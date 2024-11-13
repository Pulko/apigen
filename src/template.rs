use std::collections::HashMap;

#[derive(Debug)]
pub struct TemplateConfig {
    pub db: String,                              // e.g., "postgres"
    pub template_paths: HashMap<String, String>, // template file paths
}

const POSTGRES_TEMPLATES: [(&str, &str); 7] = [
    ("main.rs", "postgres/main.rs.tera"),
    ("entity.rs", "postgres/api/entity.rs.tera"),
    ("mod.rs", "postgres/api/mod.rs.tera"),
    ("Cargo.toml", "postgres/Cargo.toml.template"),
    (".gitignore", "postgres/.gitignore.template"),
    ("docker-compose.yml", "postgres/docker-compose.yml.template"),
    (".env", "postgres/.env.template"),
];

const SUPPORTED_DBS: [&str; 1] = ["postgres"];

impl TemplateConfig {
    pub fn new(db: &str) -> Self {
        let mut template_paths = HashMap::new();

        let db_name;

        if db.is_empty() {
            db_name = "postgres";
        } else {
            db_name = db;
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
            template_paths,
        }
    }

    pub fn is_valid(&self) -> bool {
        SUPPORTED_DBS.contains(&self.db.as_str())
    }

    pub fn get_supported_config_message(&self) -> String {
        format!(
            "Supported configurations:\n\n\ndatabases:\n {:?}\n\n\n\nGot invalid configuration: db: {}",
            SUPPORTED_DBS.join(",\n"),
            self.db,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_config_postgres() {
        let config = TemplateConfig::new("postgres");
        assert_eq!(config.db, "postgres");
        assert!(config.is_valid());
    }

    #[test]
    fn test_template_config_invalid() {
        let config = TemplateConfig::new("unknown_db");
        assert!(!config.is_valid());
    }
}
