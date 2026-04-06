# LLM Contract Enforcement for architecture.md (v0.0.16)

## 1. Problem Definition

Typical flow:

```rust
let prompt = build_prompt(...);
let response = llm.generate(prompt);
write_file("architecture.md", response);
```

### Issues
- Free-form output
- No contract enforcement
- No validation
- Silent failure

---

## 2. Core Principle

> Treat the LLM as a **contract satisfier**, not a text generator.

---

## 3. Three-Stage Enforcement Model

### 3.1 Schema Lock (Force Structured Output)

#### Required Output Format (JSON)

```json
{
  "systems": [
    {
      "name": "RenderSystem",
      "inputs": ["world_state"],
      "outputs": ["frame_buffer"],
      "success": "frame_time <= 16ms",
      "failure": "frame_time > 33ms"
    }
  ]
}
```

---

### 3.2 Rust Validation (Hard Gate)

```rust
#[derive(Deserialize)]
struct Contract {
    name: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
    success: String,
    failure: String,
}
```

#### Validation Logic

```rust
fn validate(contract: &Contract) -> Result<(), Error> {
    if contract.inputs.is_empty() {
        return Err(Error::MissingInputs);
    }
    if contract.outputs.is_empty() {
        return Err(Error::MissingOutputs);
    }
    if contract.success.is_empty() {
        return Err(Error::MissingSuccess);
    }
    if contract.failure.is_empty() {
        return Err(Error::MissingFailure);
    }
    Ok(())
}
```

---

### 3.3 Auto-Retry with Feedback

```rust
loop {
    let response = llm.generate(prompt);

    match parse_and_validate(response) {
        Ok(valid) => break valid,
        Err(e) => {
            prompt = format!(
                "Your previous output is invalid: {:?}\nFix it.\n{}",
                e, original_prompt
            );
        }
    }
}
```

---

## 4. Critical Enforcement Rules

### 4.1 Ban Natural Language

```
DO NOT output markdown.
DO NOT explain.
ONLY return valid JSON.
```

### 4.2 Enforce Measurable Conditions

```
success must be measurable (latency, count, boolean)
failure must be the boundary or inverse condition
```

### 4.3 Minimum System Count

```
You must define at least N systems.
```

---

## 5. architecture.md as Derived Artifact

### Pipeline

```
LLM → JSON Contracts
      ↓
Rust Validation
      ↓
Markdown Rendering
```

---

### Renderer Example

```rust
fn render_md(contracts: &[Contract]) -> String {
    contracts.iter().map(|c| format!(
        "## {}\n- Inputs: {:?}\n- Outputs: {:?}\n- Success: {}\n- Failure: {}\n",
        c.name, c.inputs, c.outputs, c.success, c.failure
    )).collect()
}
```

---

## 6. Graph Validation (Advanced)

```rust
fn validate_graph(systems: &[Contract]) -> Result<()> {
    let produced: HashSet<_> =
        systems.iter().flat_map(|s| &s.outputs).collect();

    for s in systems {
        for input in &s.inputs {
            if !produced.contains(input) {
                return Err(Error::UnresolvedInput(input.clone()));
            }
        }
    }
    Ok(())
}
```

---

## 7. Final Architecture

```
LLM (JSON only)
  ↓
Parser (serde)
  ↓
Validator (contract rules)
  ↓
Graph Validator
  ↓
Renderer (Markdown)
```

---

## 8. Conclusion

> The bottleneck is not the LLM — it is the absence of validation.
