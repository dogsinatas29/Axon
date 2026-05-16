
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Find get_semantic_risks
start_idx = -1
for i, line in enumerate(lines):
    if "async fn get_semantic_risks" in line:
        start_idx = i
        break

if start_idx == -1:
    sys.exit(1)

# Find the end of the function
end_idx = -1
brace_count = 0
for i in range(start_idx, len(lines)):
    brace_count += lines[i].count('{')
    brace_count -= lines[i].count('}')
    if brace_count == 0 and '{' in "".join(lines[start_idx:i+1]):
        end_idx = i + 1
        break

if start_idx != -1 and end_idx != -1:
    # Aggressive cleanup: return empty if any pending approval exists to ensure "Only One" card
    clean_func = [
        "async fn get_semantic_risks(State(daemon): State<Arc<Daemon>>) -> Json<serde_json::Value> {\n",
        "    let mut risks: Vec<serde_json::Value> = Vec::new();\n",
        "    let root_path = std::env::current_dir().unwrap_or_default();\n",
        "    let mut stack = vec![root_path];\n",
        "    let mut visited_count = 0;\n",
        "\n",
        "    let mut pending_approval_exists = false;\n",
        "    let mut temp_risks = Vec::new();\n",
        "\n",
        "    while let Some(path) = stack.pop() {\n",
        "        visited_count += 1;\n",
        "        if visited_count > 100 { break; }\n",
        "        \n",
        "        let approval_file = path.join(\".axon_approval_pending\");\n",
        "        if approval_file.exists() {\n",
        "             pending_approval_exists = true;\n",
        "             break; // Priority: If any approval exists, we don't show semantic risks\n",
        "        }\n",
        "\n",
        "        if let Some(ir) = crate::intelligence::decision::load_project_ir(\u0026path.to_string_lossy()) {\n",
        "            let extractor = crate::intelligence::semantic_debugger::SemanticRiskExtractor::new(\u0026path.to_string_lossy());\n",
        "            let extracted = extractor.extract_risks(\u0026ir).await;\n",
        "            for risk in extracted.risks {\n",
        "                let mut r_val = serde_json::to_value(risk).unwrap_or_default();\n",
        "                if let Some(obj) = r_val.as_object_mut() {\n",
        "                    let component = obj.get(\"component\").and_then(|v| v.as_str()).unwrap_or(\"\");\n",
        "                    if component == \".\" || component.is_empty() { continue; }\n",
        "                    if obj.get(\"risk_id\").is_none() {\n",
        "                         let rid = obj.get(\"id\").and_then(|v| v.as_str()).unwrap_or(\"unknown_risk\");\n",
        "                         obj.insert(\"risk_id\".to_string(), serde_json::Value::String(rid.to_string()));\n",
        "                    }\n",
        "                    temp_risks.push(r_val);\n",
        "                }\n",
        "            }\n",
        "        }\n",
        "\n",
        "        if let Ok(entries) = std::fs::read_dir(\u0026path) {\n",
        "            for entry in entries.flatten() {\n",
        "                if entry.path().is_dir() {\n",
        "                    let name = entry.file_name();\n",
        "                    if !name.to_string_lossy().starts_with('.') \u0026\u0026 name != \"target\" { stack.push(entry.path()); }\n",
        "                }\n",
        "            }\n",
        "        }\n",
        "    }\n",
        "\n",
        "    if !pending_approval_exists {\n",
        "        risks = temp_risks;\n",
        "    }\n",
        "\n",
        "    Json(serde_json::json!({ \"risks\": risks, \"decisions\": [], \"is_sealed\": false }))\n",
        "}\n"
    ]
    lines[start_idx:end_idx] = clean_func
    with open(path, "w") as f:
        f.writelines(lines)
    print("Successfully implemented Only-One logic in server.rs")
else:
    print(f"Could not find start/end: {start_idx}, {end_idx}")
