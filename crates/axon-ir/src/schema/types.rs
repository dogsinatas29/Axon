use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum FileAuthority {
    Immutable,
    ValidatorOwned,
    GeneratorPatchable,
    HumanOwned,
}

impl Default for FileAuthority {
    fn default() -> Self {
        Self::GeneratorPatchable
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchRegion {
    pub id: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OwnershipMetadata {
    pub authority: FileAuthority,
    #[serde(default)]
    pub validator_locked: bool,
    #[serde(default)]
    pub patchable_regions: Vec<PatchRegion>,
}

impl OwnershipMetadata {
    pub fn immutable() -> Self {
        Self {
            authority: FileAuthority::Immutable,
            validator_locked: true,
            patchable_regions: Vec::new(),
        }
    }

    pub fn validator_owned() -> Self {
        Self {
            authority: FileAuthority::ValidatorOwned,
            validator_locked: true,
            patchable_regions: Vec::new(),
        }
    }

    pub fn generator_patchable() -> Self {
        Self {
            authority: FileAuthority::GeneratorPatchable,
            validator_locked: false,
            patchable_regions: Vec::new(),
        }
    }

    pub fn human_owned() -> Self {
        Self {
            authority: FileAuthority::HumanOwned,
            validator_locked: false,
            patchable_regions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    C,
    Cpp,
    Rust,
    Python,
}

impl Default for Language {
    fn default() -> Self {
        Self::C
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Generic,
    Win32,
    Linux,
}

impl Default for Platform {
    fn default() -> Self {
        Self::Generic
    }
}

// v0.0.32: Runtime Subsystem - this is the runtime constitution that governs entry point, linker, and validation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Subsystem {
    Console,
    WindowsGui,
    Posix,
    Gtk4,
}

impl Default for Subsystem {
    fn default() -> Self {
        Self::Console
    }
}

// v0.0.32: Entry point type - runtime identity marker
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EntrypointType {
    Main,
    WinMain,
    WWinMain,
}

impl Default for EntrypointType {
    fn default() -> Self {
        Self::Main
    }
}

impl EntrypointType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "main" | "wmain" => Self::Main,
            "winmain" => Self::WinMain,
            "wwinmain" | "wwinmain16" | "wwinmainw" | "wwinmainunicode" | "wwinmaina" | "wwinmainansi" => Self::WWinMain,
            _ => Self::Main,
        }
    }

    pub fn canonical_name(&self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::WinMain => "WinMain",
            Self::WWinMain => "wWinMain",
        }
    }

    pub fn canonical_signature(&self, lang: Language) -> String {
        match self {
            Self::Main => match lang {
                Language::Rust => "fn main()".to_string(),
                Language::Python => "def main()".to_string(),
                _ => "int main(void)".to_string(),
            },
            Self::WinMain => "int WINAPI WinMain(HINSTANCE, HINSTANCE, LPSTR, int)".to_string(),
            Self::WWinMain => "int WINAPI wWinMain(HINSTANCE, HINSTANCE, PWSTR, int)".to_string(),
        }
    }

    pub fn canonical_file(&self, lang: Language) -> String {
        let ext = match lang {
            Language::Cpp | Language::Rust => "cpp",
            _ => "c",
        };
        match self {
            Self::Main => format!("src/main.{}", ext),
            Self::WinMain => format!("src/winmain.{}", ext),
            Self::WWinMain => format!("src/winmain.{}", ext),
        }
    }
}

// v0.0.32: Runtime model - replaces Option<String>
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeModel {
    Console,
    Win32Gui,
    EventDriven,
    Gtk4,
}

impl Default for RuntimeModel {
    fn default() -> Self {
        Self::Console
    }
}

// v0.0.32: Win32-specific component taxonomy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Win32ComponentType {
    Win32WindowClass,
    Win32WndProc,
    Win32MessageLoop,
    Win32Resource,
    Win32Dialog,
    LuaRuntime,
}

impl Default for Win32ComponentType {
    fn default() -> Self {
        Self::Win32WindowClass
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: u64,
    pub kind: String,
    pub target: String,
    pub condition: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIR {
    #[serde(default)]
    pub node_mapping: BTreeMap<String, String>,
    pub components: BTreeMap<String, Component>,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    #[serde(skip)]
    pub constraint_ids: std::collections::HashSet<u64>,
    #[serde(default)]
    pub thought: Option<String>,
    #[serde(default)]
    pub language: Language, // v0.0.31: Semantic root language declaration
    #[serde(default)]
    pub platform: Platform, // v0.0.31.14: Platform runtime personality declaration
    #[serde(default)]
    pub subsystem: Subsystem, // v0.0.32: Runtime constitution (Console / WindowsGui / Posix)
    #[serde(default)]
    pub entrypoint_type: EntrypointType, // v0.0.32: Runtime identity marker
    #[serde(default)]
    pub runtime_model: RuntimeModel, // v0.0.32: Replaces Option<String>
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ComponentTier {
    Core,       // Tier 0: Critical for project integrity
    Optional,   // Tier 1: Bonus feature, can be pruned if failing
    Experimental, // Tier 2: Sandbox feature
}

impl Default for ComponentTier {
    fn default() -> Self {
        Self::Core
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    ProjectModule,
    SystemLibrary,
    ExternalRuntime,
    Win32WindowClass,
    Win32WndProc,
    Win32MessageLoop,
    Win32Resource,
    Win32Dialog,
    LuaRuntime,
}

impl Default for ComponentType {
    fn default() -> Self {
        Self::ProjectModule
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub file_path: String,
    pub functions: BTreeMap<String, Function>,
    pub imports: BTreeSet<String>,
    #[serde(default)]
    pub associated_files: Vec<String>,
    #[serde(default)]
    pub is_entrypoint: bool,
    #[serde(default)]
    pub data_models: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>, // v0.0.28: Generic metadata
    #[serde(default)]
    pub allowed_includes: BTreeSet<String>, // v0.0.28: Dependency discipline
    #[serde(default)]
    pub forbidden_includes: BTreeSet<String>,
    #[serde(default)]
    pub forbidden_symbols: BTreeSet<String>, // v0.0.28: Logic isolation
    #[serde(default)]
    pub tier: ComponentTier, // v0.0.29: Criticality level
    #[serde(default = "default_true")]
    pub is_blocking: bool, // v0.0.29: Whether failure blocks the whole factory
    #[serde(default)]
    pub locked: bool, // v0.0.30: SSOT physical seal status
    #[serde(default)]
    pub component_type: ComponentType, // v0.0.31.20: Semantic classification
    // v0.0.32: Win32 Runtime Ontology
    #[serde(default)]
    pub subsystem: Option<Subsystem>, // Per-component subsystem override
    #[serde(default)]
    pub dll_imports: BTreeSet<String>, // e.g. user32, gdi32, kernel32
    // v0.0.31.30: FileAuthority - rewrite sovereignty (P0)
    #[serde(default)]
    pub ownership: OwnershipMetadata,
}

pub fn default_true() -> bool { true }


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub signature: String,
    pub dependencies: BTreeSet<String>,
    pub body_hash: Option<u64>,
    #[serde(default)]
    pub locked: bool, // v0.0.30: SSOT physical seal status
}

impl ProjectIR {
    pub fn new() -> Self {
        Self {
            node_mapping: BTreeMap::new(),
            components: BTreeMap::new(),
            constraints: Vec::new(),
            constraint_ids: std::collections::HashSet::new(),
            thought: None,
            language: Language::C,
            platform: Platform::Generic,
            subsystem: Subsystem::Console,
            entrypoint_type: EntrypointType::Main,
            runtime_model: RuntimeModel::Console,
        }
    }

    pub fn from_md(md: &str) -> Option<Self> {
        let start_tag = "<!-- AXON:SPEC:COMPONENTS";
        let end_tag = "-->";

        if let Some(start_idx) = md.find(start_tag) {
            let json_start = start_idx + start_tag.len();
            if let Some(end_idx) = md[json_start..].find(end_tag) {
                let json_str = md[json_start..json_start + end_idx].trim();

                #[derive(Deserialize)]
                struct Components {
                    #[serde(default)]
                    node_mapping: BTreeMap<String, String>,
                    components: Vec<RawComponent>
                }
                #[derive(Deserialize)]
                struct RawComponent {
                    file: String,
                    name: String,
                    symbols: Vec<String>,
                    #[serde(rename = "type")] _type: String,
                    #[serde(default)]
                    tier: ComponentTier,
                    #[serde(default = "default_true")]
                    is_blocking: bool,
                }

                if let Ok(raw) = serde_json::from_str::<Components>(json_str) {
                    let mut components = BTreeMap::new();
                    for c in raw.components {
                        let mut functions = BTreeMap::new();
                        for s in c.symbols {
                            functions.insert(s.clone(), Function {
                                name: s.clone(),
                                signature: format!("{}()", s),
                                dependencies: BTreeSet::new(),
                                body_hash: None,
                                locked: false,
                            });
                        }

                        // Determine component type and apply auto-classification
                        let mut comp_type = match c._type.to_lowercase().as_str() {
                            "system_library" | "system" => ComponentType::SystemLibrary,
                            "external_runtime" | "external" => ComponentType::ExternalRuntime,
                            _ => ComponentType::ProjectModule,
                        };

                        let file_lower = c.file.to_lowercase();
                        let base_name = std::path::Path::new(&file_lower)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("");
                        let name_lower = c.name.to_lowercase();
                        let system_libs = ["user32", "gdi32", "kernel32", "shell32", "comdlg32", "gdi"];
                        if system_libs.contains(&base_name) || system_libs.contains(&name_lower.as_str()) {
                            comp_type = ComponentType::SystemLibrary;
                        }

                        let canonical_key = crate::canonicalizer::canonicalize_path(&c.file);
                        let comp_name = c.name.clone();
                        components.insert(canonical_key.clone(), Component {
                            name: comp_name.clone(),
                            file_path: c.file,
                            functions,
                            imports: BTreeSet::new(),
                            associated_files: Vec::new(),
                            is_entrypoint: false,
                            data_models: Vec::new(),
                            metadata: BTreeMap::new(),
                            allowed_includes: BTreeSet::new(),
                            forbidden_includes: BTreeSet::new(),
                            forbidden_symbols: BTreeSet::new(),
                            tier: c.tier,
                            is_blocking: c.is_blocking,
                            locked: false,
                            component_type: comp_type,
                            subsystem: None,
                            dll_imports: BTreeSet::new(),
                            ownership: OwnershipMetadata::generator_patchable(),
                        });
                        tracing::debug!("[IR_REGISTER] key={} name={} type={:?}", canonical_key, comp_name, comp_type);
                    }

                    let mut constraints = Vec::new();
                    let constraint_tag = "<!-- AXON:CONSTRAINTS";
                    if let Some(c_start) = md.find(constraint_tag) {
                        let c_json_start = c_start + constraint_tag.len();
                        if let Some(c_end) = md[c_json_start..].find(end_tag) {
                            let c_json_str = md[c_json_start..c_json_start + c_end].trim();
                            if let Ok(c_list) = serde_json::from_str::<Vec<Constraint>>(c_json_str) {
                                constraints = c_list;
                            }
                        }
                    }

                    let mut language = Language::C;
                    let mut subsystem = Subsystem::Console;
                    let mut entrypoint_type = EntrypointType::Main;
                    let mut runtime_model = RuntimeModel::Console;

                    let md_lower = md.to_lowercase();
                    if md_lower.contains("language: rust") || md_lower.contains("language:rust") {
                        language = Language::Rust;
                    } else if md_lower.contains("language: python") || md_lower.contains("language:python") {
                        language = Language::Python;
                    } else if md_lower.contains("language: cpp") || md_lower.contains("language:cpp") {
                        language = Language::Cpp;
                    }

                    let mut platform = Platform::Generic;
                    if md_lower.contains("platform: win32")
                        || md_lower.contains("platform:win32")
                        || md_lower.contains("subsystem: windows")
                        || md_lower.contains("subsystem:windows")
                        || md_lower.contains("win32")
                    {
                        platform = Platform::Win32;
                        // v0.0.32: Auto-promote for Win32 GUI projects
                        if md_lower.contains("gui")
                            || md_lower.contains("winnt")
                            || md_lower.contains("winsdk")
                        {
                            subsystem = Subsystem::WindowsGui;
                            entrypoint_type = EntrypointType::WWinMain;
                            runtime_model = RuntimeModel::Win32Gui;
                            language = Language::Cpp; // Win32 GUI auto-requires C++
                        }
                    }

                    return Some(ProjectIR {
                        node_mapping: raw.node_mapping,
                        components,
                        constraints,
                        constraint_ids: std::collections::HashSet::new(),
                        thought: None,
                        language,
                        platform,
                        subsystem,
                        entrypoint_type,
                        runtime_model,
                    });
                }
            }
        }
        None
    }

    pub fn get_component(&self, canonical_path: &str) -> Option<&Component> {
        let key = crate::canonicalizer::canonicalize_path(canonical_path);
        self.components.get(&key)
    }

    pub fn get_all_keys(&self) -> Vec<String> {
        self.components.keys().cloned().collect()
    }
}

impl Default for ProjectIR {
    fn default() -> Self {
        Self::new()
    }
}