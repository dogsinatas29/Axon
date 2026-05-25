use crate::schema::Language;

#[derive(Debug, Clone)]
pub struct SemanticSpec {
    pub language: Language,
    pub extracted_terms: Vec<String>,
    pub component_names: Vec<String>,
    pub file_paths: Vec<String>,
    pub build_systems: Vec<String>,
    pub task_vocabulary: Vec<String>,
}

impl SemanticSpec {
    pub fn from_llm_json(json: &str) -> anyhow::Result<Self> {
        use serde_json::Value;

        let value: Value = serde_json::from_str(json)?;

        let language = value
            .get("language")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "c" | "C" => Language::C,
                "cpp" | "c++" | "Cpp" => Language::Cpp,
                "rust" | "Rust" => Language::Rust,
                "python" | "Python" => Language::Python,
                _ => Language::C,
            })
            .unwrap_or(Language::C);

        let mut extracted_terms = Vec::new();
        let mut component_names = Vec::new();
        let mut file_paths = Vec::new();
        let mut build_systems = Vec::new();
        let mut task_vocabulary = Vec::new();

        if let Some(components) = value.get("components").and_then(|v| v.as_array()) {
            for comp in components {
                if let Some(name) = comp.get("name").and_then(|v| v.as_str()) {
                    component_names.push(name.to_string());
                }
                if let Some(file) = comp.get("file").and_then(|v| v.as_str()) {
                    file_paths.push(file.to_string());
                    extracted_terms.push(file.to_string());  // file path also for vocabulary check
                    if let Some(ext) = std::path::Path::new(file).extension() {
                        extracted_terms.push(format!(".{}", ext.to_string_lossy()));  // extension check
                    }
                }
                if let Some(kind) = comp.get("kind").and_then(|v| v.as_str()) {
                    task_vocabulary.push(kind.to_string());
                    extracted_terms.push(kind.to_string());
                }
            }
        }

        if let Some(build) = value.get("build_system").and_then(|v| v.as_str()) {
            build_systems.push(build.to_string());
        }

        if let Some(node_mapping) = value.get("node_mapping").and_then(|v| v.as_object()) {
            for (key, value) in node_mapping {
                extracted_terms.push(key.clone());
                if let Some(v_str) = value.as_str() {
                    extracted_terms.push(v_str.to_string());
                }
            }
        }

        Ok(SemanticSpec {
            language,
            extracted_terms,
            component_names,
            file_paths,
            build_systems,
            task_vocabulary,
        })
    }
}