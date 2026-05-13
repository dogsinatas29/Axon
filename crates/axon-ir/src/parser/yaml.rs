use crate::schema::ProjectIR;
use anyhow::Result;

pub fn parse_yaml(_input: &str) -> Result<Option<ProjectIR>> {
    anyhow::bail!("YAML parser not yet implemented - placeholder for future")
}

pub fn to_yaml(_ir: &ProjectIR) -> Result<String> {
    anyhow::bail!("YAML serializer not yet implemented")
}