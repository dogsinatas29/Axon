use crate::schema::ProjectIR;
use anyhow::Result;

pub fn parse_json(input: &str) -> Result<Option<ProjectIR>> {
    let ir: ProjectIR = serde_json::from_str(input)?;
    tracing::info!("[IR_PARSE] JSON parsed: {} components", ir.components.len());
    Ok(Some(ir))
}

pub fn serialize(ir: &ProjectIR) -> Result<String> {
    Ok(serde_json::to_string_pretty(ir)?)
}