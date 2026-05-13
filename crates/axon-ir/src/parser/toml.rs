use crate::schema::ProjectIR;
use anyhow::Result;

pub fn parse_toml(_input: &str) -> Result<Option<ProjectIR>> {
    anyhow::bail!("TOML parser not yet implemented - placeholder for future")
}

pub fn to_toml(_ir: &ProjectIR) -> Result<String> {
    anyhow::bail!("TOML serializer not yet implemented")
}