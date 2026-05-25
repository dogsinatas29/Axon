use super::ontology::get_ontology;
use super::spec::SemanticSpec;
use super::violation::SemanticViolation;

pub struct SpecSemanticValidator;

impl SpecSemanticValidator {
    pub fn validate(spec: &SemanticSpec) -> Result<(), SemanticViolation> {
        let ontology = get_ontology(spec.language);

        for term in &spec.extracted_terms {
            let term_lower = term.to_lowercase();
            for forbidden in ontology.forbidden_vocabulary() {
                if term_lower.contains(&forbidden.to_lowercase()) {
                    return Err(SemanticViolation::ForbiddenVocabulary {
                        term: term.clone(),
                        language: spec.language,
                    });
                }
            }
        }

        for sys in &spec.build_systems {
            let allowed = ontology.allowed_build_systems();
            if !allowed.is_empty() && !allowed.iter().any(|a| sys.to_lowercase().contains(&a.to_lowercase())) {
                return Err(SemanticViolation::InvalidBuildSystem {
                    system: sys.clone(),
                    language: spec.language,
                });
            }
        }

        for task in &spec.task_vocabulary {
            let allowed = ontology.allowed_task_vocabulary();
            if !allowed.is_empty() {
                let task_lower = task.to_lowercase();
                let valid = allowed.iter().any(|a| task_lower.contains(&a.to_lowercase()));
                if !valid {
                    return Err(SemanticViolation::InvalidTaskVocabulary {
                        task: task.clone(),
                        language: spec.language,
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_forbids_header() {
        let json = r#"{
            "language": "rust",
            "components": [{"name": "foo", "kind": "HeaderDecl"}]
        }"#;
        let spec = SemanticSpec::from_llm_json(json).unwrap();
        let result = SpecSemanticValidator::validate(&spec);
        assert!(result.is_err());
    }

    #[test]
    fn test_c_allows_header() {
        let json = r#"{
            "language": "c",
            "components": [{"name": "foo", "kind": "HeaderDecl"}]
        }"#;
        let spec = SemanticSpec::from_llm_json(json).unwrap();
        let result = SpecSemanticValidator::validate(&spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_python_forbids_cargo() {
        let json = r#"{
            "language": "python",
            "build_system": "cargo",
            "components": []
        }"#;
        let spec = SemanticSpec::from_llm_json(json).unwrap();
        let result = SpecSemanticValidator::validate(&spec);
        assert!(result.is_err());
    }
}