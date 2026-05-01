use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Spec {
    pub components: Vec<ComponentSpec>,
}

#[derive(Debug, Clone)]
pub struct ComponentSpec {
    pub name: String,
    pub functions: Vec<FunctionSpec>,
}

#[derive(Debug, Clone)]
pub struct FunctionSpec {
    pub name: String,
    pub signature: String,
}

pub fn parse_architecture_md(input: &str) -> Result<Spec, String> {
    if let Some(json) = extract_json_block(input) {
        if let Ok(spec) = parse_json_spec(&json) {
            return Ok(spec);
        }
    }

    parse_markdown_spec(input)
}

fn extract_json_block(input: &str) -> Option<String> {
    let start_marker = "AXON:SPEC:COMPONENTS";
    let start = input.find(start_marker)?;

    let json_start = input[start..].find('{')? + start;
    let json_end = input[json_start..].rfind('}')? + json_start;

    Some(input[json_start..=json_end].to_string())
}

fn parse_json_spec(json: &str) -> Result<Spec, String> {
    #[derive(Deserialize)]
    struct RawSpec {
        components: Vec<RawComponent>,
    }

    #[derive(Deserialize)]
    struct RawComponent {
        name: String,
        functions: Option<Vec<RawFunction>>, // Compatibility for simple lists
        symbols: Option<Vec<String>>,        // Legacy compatibility
    }

    #[derive(Deserialize)]
    struct RawFunction {
        name: String,
        signature: String,
    }

    let parsed: RawSpec = serde_json::from_str(json)
        .map_err(|e| e.to_string())?;

    Ok(Spec {
        components: parsed.components.into_iter().map(|c| {
            let functions = if let Some(fs) = c.functions {
                fs.into_iter().map(|f| FunctionSpec {
                    name: f.name,
                    signature: f.signature,
                }).collect()
            } else if let Some(syms) = c.symbols {
                syms.into_iter().map(|s| FunctionSpec {
                    name: s.clone(),
                    signature: format!("{}()", s),
                }).collect()
            } else {
                vec![]
            };

            ComponentSpec {
                name: c.name,
                functions,
            }
        }).collect(),
    })
}

fn parse_markdown_spec(input: &str) -> Result<Spec, String> {
    let mut components = Vec::new();
    let mut current_component: Option<ComponentSpec> = None;

    for line in input.lines() {
        let line = line.trim();

        if line.starts_with("## Component:") {
            if let Some(c) = current_component.take() {
                components.push(c);
            }

            let name = line.replace("## Component:", "").trim().to_string();

            current_component = Some(ComponentSpec {
                name,
                functions: Vec::new(),
            });
        }
        else if line.starts_with("- ") {
            if let Some(ref mut comp) = current_component {
                let sig = line.trim_start_matches("- ").trim().to_string();
                let name = extract_fn_name(&sig);

                comp.functions.push(FunctionSpec {
                    name,
                    signature: sig,
                });
            }
        }
    }

    if let Some(c) = current_component {
        components.push(c);
    }

    if components.is_empty() {
        return Err("No components found in architecture.md".to_string());
    }

    Ok(Spec { components })
}

fn extract_fn_name(signature: &str) -> String {
    signature
        .split('(')
        .next()
        .unwrap_or(signature)
        .to_string()
}
