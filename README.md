# RAPIDO

## Overview

This project is a Rust-based API generator that allows you to generate a complete API based on a given schema in JSON format. It is designed to support multiple backends and database configurations, currently supporting PostgreSQL templates.

## Features

- Generates API folders with schema, entities, and configuration files.
- Supports different backends and databases through templates (PostgreSQL is supported out of the box).
- Automatically creates folder structures and essential files (`Cargo.toml`, `.gitignore`, `Dockerfile`, etc.).
- Extensible with multiple filters like `capitalize_first_letter`, `pluralize`, and `diesel_type` for template rendering.

## Prerequisites

- **Rust**: Make sure Rust is installed. You can get it from [rust-lang.org](https://www.rust-lang.org/tools/install).
- **Cargo**: Cargo should be installed as part of the Rust toolchain.

To check if Rust is installed, run:

```bash
rustc --version
```

## Usage

1. Clone the repository:

   ```bash
   git clone <your-repo-url>
   cd api_generator
   ```

2. Prepare your API schema in JSON format. An example schema:

   ```json
   {
     "entities": [
       {
         "name": "User",
         "fields": [
           { "name": "id", "field_type": "u32" },
           {
             "name": "username",
             "field_type": "String"
           }
         ]
       },
       {
         "name": "Post",
         "fields": [
           { "name": "id", "field_type": "u32" },
           {
             "name": "title",
             "field_type": "String"
           },
           {
             "name": "body",
             "field_type": "String"
           },
           {
             "name": "author_id",
             "field_type": "u32"
           }
         ]
       }
     ]
   }
   ```

3. Run the RAPIDO:

   ```bash
   cargo run -- '<api_schema_json>'
   ```

   Replace `<api_schema_json>` with your actual JSON schema. For example:

   ```bash
   cargo run -- '{"entities":[{"name":"User","fields":[{"name":"id","field_type":"u32"},{"name":"username","field_type":"String"}]}]}'
   ```

4. The generated API will be saved in the `output` directory. The output folder will contain:
   - A generated `Cargo.toml` file
   - Source code for the API, including routes and schema
   - Other configuration files such as `.gitignore` and `Dockerfile`.

## Configuration

The project uses a `TemplateConfig` to load template files depending on the selected database backend. By default, it uses PostgreSQL templates located in the `templates/postgres/` directory.

If you want to extend the project to support more databases or backends, you can:

1. Add new templates in the `templates/<backend>/` directory.
2. Adjust the `TemplateConfig` to point to your new templates.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
