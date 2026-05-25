pub mod ontology;
pub mod spec;
pub mod validator;
pub mod violation;

pub use ontology::{get_ontology, SemanticOntology};
pub use spec::SemanticSpec;
pub use validator::SpecSemanticValidator;
pub use violation::{SemanticViolation, SemanticDiagnostic, DiagnosticSeverity, DiagnosticCategory, CapabilityLock};