
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Update get_semantic_risks to fetch real logs from thread posts
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
        "                 \"message\": \"Bootstrap approval required\",\n",
        "                 \"cause\": \"New project design requires sovereign authorization.\",\n",
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
        "            // v0.0.30: Deep History Mining - find the last ACTUAL rejection log\n",
        "            let last_rejection_post = posts.iter().rev().find(|p| {\n",
        "                if let Some(fb) = \u0026p.error_feedback {\n",
        "                    !fb.contains(\"BOSS_OVERRIDE\") \u0026\u0026 !fb.is_empty()\n",
        "                } else { false }\n",
        "            });\n",
        "\n",
        "            let raw_log = last_rejection_post.and_then(|p| p.error_feedback.clone()).unwrap_or_else(|| task.error_feedback.clone().unwrap_or_else(|| \"Unknown Failure\".to_string()));\n",
        "            let last_code = posts.iter().rev().find(|p| p.full_code.is_some()).and_then(|p| p.full_code.clone());\n",
        "            let recommendation = last_rejection_post.and_then(|p| p.senior_comment.clone()).unwrap_or_else(|| \"설계도(IR)를 참조하여 함수명과 스키마를 다시 확인하십시오.\".to_string());\n",
        "            \n",
        "            let mut actor = \"System Gate\";\n",
        "            if raw_log.contains(\"SENIOR_REJECT\") { actor = \"Senior AI (Claude/DeepSeek)\"; }\n",
        "            else if raw_log.contains(\"VALIDATOR_FAIL\") { actor = \"Validator Engine (Rust Native)\"; }\n",
        "            else if raw_log.contains(\"cmake\") || raw_log.contains(\"error:\") { actor = \"Compiler (Clang/GCC)\"; }\n",
        "\n",
        "            risks.push(serde_json::json!({\n",
        "                \"risk_id\": format!(\"rejection_limit_{}\", task.id),\n",
        "                \"kind\": \"ImplementationFail\",\n",
        "                \"level\": \"Critical\",\n",
        "                \"target\": task.title,\n",
        "                \"actor\": actor,\n",
        "                \"cause\": raw_log,\n",
        "                \"recommendation\": recommendation,\n",
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
print("Successfully patched server.rs with Deep Rejection History Mining")
