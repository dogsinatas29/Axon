
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Update get_semantic_risks for precise highlighting and actor identification
start_idx = -1
for i, line in enumerate(lines):
    if "async fn get_semantic_risks" in line:
        start_idx = i
        break

end_idx = -1
brace_count = 0
for i in range(start_idx, len(lines)):
    brace_count += lines[i].count('{')
    brace_count -= lines[i].count('}')
    if brace_count == 0 and '{' in "".join(lines[start_idx:i+1]):
        end_idx = i + 1
        break

if start_idx != -1 and end_idx != -1:
    new_func = [
        "async fn get_semantic_risks(State(daemon): State<Arc<Daemon>>) -> Json<serde_json::Value> {\n",
        "    let mut risks: Vec<serde_json::Value> = Vec::new();\n",
        "    let root_path = std::env::current_dir().unwrap_or_default();\n",
        "    let mut stack = vec![root_path];\n",
        "    let mut visited_count = 0;\n",
        "\n",
        "    while let Some(path) = stack.pop() {\n",
        "        visited_count += 1;\n",
        "        if visited_count > 100 { break; }\n",
        "        let approval_file = path.join(\".axon_approval_pending\");\n",
        "        if approval_file.exists() {\n",
        "             risks.push(serde_json::json!({\n",
        "                 \"risk_id\": \"pending_approval\",\n",
        "                 \"kind\": \"Bootstrap\",\n",
        "                 \"level\": \"Critical\",\n",
        "                 \"target\": \"Factory Gateway\",\n",
        "                 \"failed_stage\": \"SpecAnalysis\",\n",
        "                 \"cause\": \"Constitutional design requires boss authorization.\",\n",
        "                 \"component\": \"BOOTSTRAP\",\n",
        "             }));\n",
        "        }\n",
        "        if let Some(ir) = crate::intelligence::decision::load_project_ir(\u0026path.to_string_lossy()) {\n",
        "            let extractor = crate::intelligence::semantic_debugger::SemanticRiskExtractor::new(\u0026path.to_string_lossy());\n",
        "            let extracted = extractor.extract_risks(\u0026ir).await;\n",
        "            for risk in extracted.risks { risks.push(serde_json::to_value(risk).unwrap()); }\n",
        "        }\n",
        "        if let Ok(entries) = std::fs::read_dir(\u0026path) {\n",
        "            for entry in entries.flatten() {\n",
        "                if entry.path().is_dir() \u0026\u0026 !entry.file_name().to_string_lossy().starts_with('.') \u0026\u0026 entry.file_name() != \"target\" { stack.push(entry.path()); }\n",
        "            }\n",
        "        }\n",
        "    }\n",
        "\n",
        "    let tasks = daemon.storage.list_all_tasks().unwrap_or_default();\n",
        "    for task in tasks {\n",
        "        if task.rework_count >= 3 {\n",
        "            let posts = daemon.storage.list_posts_by_thread(\u0026task.id).unwrap_or_default();\n",
        "            let error_post = posts.iter().rev().find(|p| p.author_id != \"BOSS\" \u0026\u0026 (p.content.to_lowercase().contains(\"error\") || p.content.to_lowercase().contains(\"reject\") || p.content.to_lowercase().contains(\"fail\")));\n",
        "            let raw_log = error_post.map(|p| p.content.clone()).unwrap_or_else(|| task.error_feedback.clone().unwrap_or_else(|| \"Unknown Failure\".to_string()));\n",
        "            let last_code = posts.iter().rev().find(|p| p.full_code.is_some()).and_then(|p| p.full_code.clone());\n",
        "            \n",
        "            let mut actor = \"Contract Verifier\";\n",
        "            let mut failed_stage = \"Contract Verification\";\n",
        "            if raw_log.contains(\"error:\") || raw_log.contains(\"cmake\") {\n",
        "                actor = \"Compiler (Clang/GCC)\";\n",
        "                failed_stage = \"Build/Linking\";\n",
        "            } else if raw_log.contains(\"SENIOR_REJECT\") || raw_log.contains(\"Review\") {\n",
        "                actor = \"Senior AI Auditor\";\n",
        "                failed_stage = \"Semantic Review\";\n",
        "            }\n",
        "\n",
        "            // Extract Line Number for Highlighting\n",
        "            let mut target_line = -1;\n",
        "            if let Some(caps) = regex::Regex::new(r\"[:\\s](\\d+)[:\\s]\").ok().and_then(|re| re.captures(\u0026raw_log)) {\n",
        "                target_line = caps.get(1).and_then(|m| m.as_str().parse::\u003Ci32\u003E().ok()).unwrap_or(-1);\n",
        "            }\n",
        "\n",
        "            // Extract Expected Contract with fuzzy matching\n",
        "            let mut expected = \"No contract found in IR\".to_string();\n",
        "            let mut detected = \"Unknown\".to_string();\n",
        "            if let Some(target) = \u0026task.target_file {\n",
        "                let root = std::env::current_dir().unwrap_or_default();\n",
        "                if let Some(ir) = crate::intelligence::decision::load_project_ir(\u0026root.to_string_lossy()) {\n",
        "                    let target_basename = target.split('/').last().unwrap_or(target).replace(\".c\", \"\").replace(\".h\", \"\");\n",
        "                    if let Some(comp) = ir.components.iter().find(|c| c.1.name.contains(\u0026target_basename)) {\n",
        "                        let funcs: Vec\u003CString\u003E = comp.1.functions.iter().map(|f| f.1.name.clone()).collect();\n",
        "                        expected = format!(\"Required Symbols: [{}]\", funcs.join(\", \"));\n",
        "                        if let Some(code) = \u0026last_code {\n",
        "                            if !funcs.is_empty() \u0026\u0026 !code.contains(\u0026funcs[0]) {\n",
        "                                detected = format!(\"Missing required symbol: {}\", funcs[0]);\n",
        "                            }\n",
        "                        }\n",
        "                    }\n",
        "                }\n",
        "            }\n",
        "\n",
        "            risks.push(serde_json::json!({\n",
        "                \"risk_id\": format!(\"rejection_limit_{}\", task.id),\n",
        "                \"kind\": \"ImplementationFail\",\n",
        "                \"level\": \"Critical\",\n",
        "                \"target\": task.title,\n",
        "                \"actor\": actor,\n",
        "                \"failed_stage\": failed_stage,\n",
        "                \"cause\": raw_log,\n",
        "                \"expected\": expected,\n",
        "                \"detected\": detected,\n",
        "                \"target_line\": target_line,\n",
        "                \"component\": task.target_file.clone().unwrap_or_else(|| \"unknown\".to_string()),\n",
        "                \"full_code\": last_code,\n",
        "                \"task_id\": task.id,\n",
        "            }));\n",
        "        }\n",
        "    }\n",
        "    Json(serde_json::json!({ \"risks\": risks }))\n",
        "}\n"
    ]
    lines[start_idx:end_idx] = new_func

with open(path, "w") as f:
    f.writelines(lines)
print("Successfully patched server.rs with Precise Highlighting and Actor ID")
