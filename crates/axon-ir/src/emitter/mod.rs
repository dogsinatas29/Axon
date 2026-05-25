use crate::schema::ProjectIR;
use std::path::Path;
use std::fs;
use std::io;

pub fn save_ir(ir: &ProjectIR, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(ir)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(path, json)?;
    tracing::info!("[IR_SAVE] saved to {:?}", path);
    Ok(())
}

pub fn load_ir(path: &Path) -> Option<ProjectIR> {
    if !path.exists() {
        tracing::warn!("[IR_LOAD] file not found: {:?}", path);
        return None;
    }

    let content = fs::read_to_string(path).ok()?;
    let ir: ProjectIR = serde_json::from_str(&content).ok()?;
    tracing::info!("[IR_LOAD] loaded {} components from {:?}", ir.components.len(), path);
    Some(ir)
}

pub fn load_ir_from_path(path_str: &str) -> Option<ProjectIR> {
    let path = Path::new(path_str);
    load_ir(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_save_load() {
        use crate::schema::{Component, Function};

        let mut ir = ProjectIR::new();
        ir.components.insert("test.c".to_string(), Component {
            name: "Test".to_string(),
            file_path: "test.c".to_string(),
            functions: std::collections::BTreeMap::new(),
            imports: std::collections::BTreeSet::new(),
            associated_files: Vec::new(),
            is_entrypoint: false,
            data_models: Vec::new(),
            metadata: std::collections::BTreeMap::new(),
            allowed_includes: std::collections::BTreeSet::new(),
            forbidden_includes: std::collections::BTreeSet::new(),
            forbidden_symbols: std::collections::BTreeSet::new(),
            tier: crate::schema::ComponentTier::Core,
            is_blocking: true,
            locked: false,
            component_type: crate::schema::ComponentType::ProjectModule,
            subsystem: None,
            dll_imports: std::collections::BTreeSet::new(),
            ownership: crate::schema::OwnershipMetadata::generator_patchable(),
        });

        let path = Path::new("/tmp/test_ir.json");
        save_ir(&ir, path).unwrap();

        let loaded = load_ir(path).unwrap();
        assert!(loaded.components.contains_key("test.c"));

        std::fs::remove_file(path).ok();
    }
}