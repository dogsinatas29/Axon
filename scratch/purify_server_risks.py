
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Find get_semantic_risks start and end
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
    # Pure, clean implementation of get_semantic_risks with deduplication
    clean_func = [
        "async fn get_semantic_risks(State(daemon): State<Arc<Daemon>>) -> Json<serde_json::Value> {\n",
        "    let mut risks: Vec<serde_json::Value> = Vec::new();\n",
        "    let root_path = std::env::current_dir().unwrap_or_default();\n",
        "    let mut stack = vec![root_path];\n",
        "    let mut visited_count = 0;\n",
        "\n",
        "    // v0.0.30: [DEDUPLICATION] Get pending projects to hide redundant semantic risks\n",
        "    let pending_projects: std::collections::HashSet<String> = daemon.storage.list_all_threads().unwrap_or_default()\n",
        "        .into_iter()\n",
        "        .filter(|t| t.status == axon_core::ThreadStatus::Draft || t.status == axon_core::ThreadStatus::Working)\n",
        "        .map(|t| t.project_id.clone())\n",
        "        .collect();\n",
        "\n",
        "    while let Some(path) = stack.pop() {\n",
        "        visited_count += 1;\n",
        "        if visited_count > 100 { break; }\n",
        "        let approval_file = path.join(\".axon_approval_pending\");\n",
        "        if approval_file.exists() {\n",
        "            if let Ok(content) = std::fs::read_to_string(\u0026approval_file) {\n",
        "                if let Ok(approval) = serde_json::from_str::<serde_json::Value>(\u0026content) {\n",
        "                    if approval[\"approved\"].as_bool() != Some(true) {\n",
        "                        let project_id = approval[\"project_id\"].as_str().unwrap_or(\"\");\n",
        "                        if !pending_projects.contains(project_id) {\n",
        "                            if let Some(extra_risks) = approval.get(\"risks\").and_then(|v| v.as_array()) {\n",
        "                                for r_val in extra_risks { risks.push(r_val.clone()); }\n",
        "                            }\n",
        "                        }\n",
        "                    }\n",
        "                }\n",
        "            }\n",
        "        }\n",
        "        if let Some(ir) = crate::intelligence::decision::load_project_ir(\u0026path.to_string_lossy()) {\n",
        "            let extractor = crate::intelligence::semantic_debugger::SemanticRiskExtractor::new(\u0026path.to_string_lossy());\n",
        "            let extracted = extractor.extract_risks(\u0026ir).await;\n",
        "            for risk in extracted.risks { risks.push(serde_json::to_value(risk).unwrap_or_default()); }\n",
        "        }\n",
        "        if let Ok(entries) = std::fs::read_dir(\u0026path) {\n",
        "            for entry in entries.flatten() {\n",
        "                if entry.path().is_dir() {\n",
        "                    let name = entry.file_name();\n",
        "                    if !name.to_string_lossy().starts_with('.') \u0026\u0026 name != \"target\" { stack.push(entry.path()); }\n",
        "                }\n",
        "            }\n",
        "        }\n",
        "    }\n",
        "    Json(serde_json::json!({ \"risks\": risks, \"decisions\": [], \"is_sealed\": false }))\n",
        "}\n"
    ]
    lines[start_idx:end_idx] = clean_func
    with open(path, "w") as f:
        f.writelines(lines)
    print("Successfully purified get_semantic_risks in server.rs")
else:
    print(f"Could not find start/end: {start_idx}, {end_idx}")
