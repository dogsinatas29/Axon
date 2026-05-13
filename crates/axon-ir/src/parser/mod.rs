use crate::schema::ProjectIR;
use anyhow::Result;

pub mod markdown;
pub mod json;
pub mod yaml;
pub mod toml;

pub fn parse_md(input: &str) -> Result<Option<ProjectIR>> {
    Ok(ProjectIR::from_md(input))
}

pub fn parse_json(input: &str) -> Result<Option<ProjectIR>> {
    match serde_json::from_str::<ProjectIR>(input) {
        Ok(ir) => Ok(Some(ir)),
        Err(_) => Ok(None),
    }
}

pub fn parse(input: &str, format: InputFormat) -> Result<Option<ProjectIR>> {
    match format {
        InputFormat::Markdown => parse_md(input),
        InputFormat::Json => parse_json(input),
        InputFormat::Yaml => yaml::parse_yaml(input),
        InputFormat::Toml => toml::parse_toml(input),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputFormat {
    Markdown,
    Json,
    Yaml,
    Toml,
}

impl InputFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" => Some(Self::Markdown),
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            _ => None,
        }
    }
}

pub fn detect_format(content: &str) -> Option<InputFormat> {
    let trimmed = content.trim();

    if trimmed.starts_with("<!-- AXON:SPEC") {
        return Some(InputFormat::Markdown);
    }

    if trimmed.starts_with('{') {
        if trimmed.contains("\"components\"") {
            return Some(InputFormat::Json);
        }
    }

    if trimmed.starts_with("---") || trimmed.contains(": ") {
        if !trimmed.starts_with('{') {
            return Some(InputFormat::Yaml);
        }
    }

    None
}