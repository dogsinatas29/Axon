
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# 1. Update get_semantic_risks to include code and better info
# 2. Update submit_semantic_decision to handle REWORK action

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
    # Improved get_semantic_risks
    new_risks_func = [
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
        "                 \"message\": \"Spec analysis complete. Boss approval required to begin manufacturing.\",\n",
        "                 \"cause\": \"Bootstrap protocol mandates human arbitration for new project specs.\",\n",
        "                 \"impact\": \"Factory remains idle until this design is authorized.\",\n",
        "                 \"recommendation\": \"Press [SEAL] to authorize the spec and activate the production line.\",\n",
        "                 \"component\": \"BOOTSTRAP\",\n",
        "             }));\n",
        "        }\n",
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
        "                    // Standardize fields for UI\n",
        "                    let msg = obj.get(\"message\").cloned().unwrap_or_else(|| obj.get(\"description\").cloned().unwrap_or(serde_json::json!(\"Unknown risk\")));\n",
        "                    obj.insert(\"message\".to_string(), msg);\n",
        "                    risks.push(serde_json::to_value(obj).unwrap());\n",
        "                }\n",
        "            }\n",
        "        }\n",
        "        if let Ok(entries) = std::fs::read_dir(\u0026path) {\n",
        "            for entry in entries.flatten() {\n",
        "                if entry.path().is_dir() \u0026\u0026 !entry.file_name().to_string_lossy().starts_with('.') \u0026\u0026 entry.file_name() != \"target\" { stack.push(entry.path()); }\n",
        "            }\n",
        "        }\n",
        "    }\n",
        "\n",
        "    // v0.0.30: [QUARANTINE_REPORTING] Scan for repetitive failures with full context\n",
        "    let tasks = daemon.storage.list_all_tasks().unwrap_or_default();\n",
        "    for task in tasks {\n",
        "        if task.rework_count >= 3 {\n",
        "            let posts = daemon.storage.list_posts(\u0026task.id).unwrap_or_default();\n",
        "            let last_code = posts.iter().rev().find(|p| p.full_code.is_some()).and_then(|p| p.full_code.clone());\n",
        "            \n",
        "            risks.push(serde_json::json!({\n",
        "                \"risk_id\": format!(\"rejection_limit_{}\", task.id),\n",
        "                \"kind\": \"ImplementationFail\",\n",
        "                \"level\": \"Critical\",\n",
        "                \"target\": task.title,\n",
        "                \"message\": format!(\"Task '{}' quarantined after 3 rejections.\", task.title),\n",
        "                \"cause\": task.error_feedback.clone().unwrap_or_else(|| \"Validator or Build Failure\".to_string()),\n",
        "                \"impact\": \"This component is blocking the factory integration gate.\",\n",
        "                \"recommendation\": \"Review the code below. Use 'REWORK with Hint' to guide the agent, or 'SEAL' to force approval.\",\n",
        "                \"component\": task.target_file.clone().unwrap_or_else(|| \"unknown\".to_string()),\n",
        "                \"full_code\": last_code,\n",
        "                \"task_id\": task.id,\n",
        "            }));\n",
        "        }\n",
        "    }\n",
        "    Json(serde_json::json!({ \"risks\": risks, \"decisions\": [], \"is_sealed\": false }))\n",
        "}\n"
    ]
    lines[start_idx:end_idx] = new_risks_func

    # 3. Update submit_semantic_decision to handle REWORK
    decide_idx = -1
    for i, line in enumerate(lines):
        if "async fn submit_semantic_decision" in line:
            decide_idx = i
            break
    
    decide_end = -1
    brace_count = 0
    for i in range(decide_idx, len(lines)):
        brace_count += lines[i].count('{')
        brace_count -= lines[i].count('}')
        if brace_count == 0 and '{' in "".join(lines[decide_idx:i+1]):
            decide_end = i + 1
            break
    
    if decide_idx != -1 and decide_end != -1:
        new_decide_func = [
            "async fn submit_semantic_decision(\n",
            "    State(daemon): State<Arc<Daemon>>,\n",
            "    Json(decision): Json<serde_json::Value>,\n",
            ") -> impl IntoResponse {\n",
            "    let risk_id = decision[\"risk_id\"].as_str().unwrap_or_default();\n",
            "    let action = decision[\"action\"].as_str().unwrap_or(\"SEAL\");\n",
            "    let comment = decision[\"comment\"].as_str().unwrap_or(\"\");\n",
            "\n",
            "    tracing::info!(\"⚖️ [BOSS_DECISION] Risk: {}, Action: {}\", risk_id, action);\n",
            "\n",
            "    if risk_id == \"pending_approval\" {\n",
            "        let root = std::env::current_dir().unwrap_or_default();\n",
            "        let mut stack = vec![root];\n",
            "        while let Some(path) = stack.pop() {\n",
            "            let approval_file = path.join(\".axon_approval_pending\");\n",
            "            if approval_file.exists() {\n",
            "                if action == \"SEAL\" || action == \"Approve\" {\n",
            "                    let _ = std::fs::remove_file(approval_file);\n",
            "                    tracing::info!(\"✅ [BOSS_APPROVED] Bootstrap gate cleared.\");\n",
            "                } else {\n",
            "                    let _ = std::fs::write(path.join(\".axon_rejected\"), \"Boss rejected bootstrap\");\n",
            "                }\n",
            "            }\n",
            "            if let Ok(entries) = std::fs::read_dir(\u0026path) {\n",
            "                for entry in entries.flatten() {\n",
            "                    if entry.path().is_dir() \u0026\u0026 !entry.file_name().to_string_lossy().starts_with('.') { stack.push(entry.path()); }\n",
            "                }\n",
            "            }\n",
            "        }\n",
            "        return StatusCode::OK.into_response();\n",
            "    }\n",
            "\n",
            "    if risk_id.starts_with(\"rejection_limit_\") {\n",
            "        let task_id = \u0026risk_id[\"rejection_limit_\".len()..];\n",
            "        if let Ok(Some(mut task)) = daemon.storage.get_task(task_id.to_string()) {\n",
            "            if action == \"SEAL\" {\n",
            "                task.status = axon_core::TaskStatus::Completed;\n",
            "                task.error_feedback = Some(format!(\"[BOSS_OVERRIDE]: {}\", comment));\n",
            "                let _ = daemon.storage.save_task(task).await;\n",
            "                tracing::info!(\"✅ [BOSS_SEALED] Task {} force-completed.\", task_id);\n",
            "            } else if action == \"REWORK\" {\n",
            "                task.rework_count = 0;\n",
            "                task.status = axon_core::TaskStatus::Pending;\n",
            "                task.senior_comment = Some(format!(\"[BOSS_HINT]: {}\", comment));\n",
            "                let _ = daemon.storage.save_task(task.clone()).await;\n",
            "                let _ = daemon.dispatcher.enqueue_task(task);\n",
            "                tracing::info!(\"🔄 [BOSS_REWORK] Task {} re-queued with hint.\", task_id);\n",
            "            } else {\n",
            "                task.status = axon_core::TaskStatus::Failed;\n",
            "                let _ = daemon.storage.save_task(task).await;\n",
            "                tracing::info!(\"🛑 [BOSS_CANCELLED] Task {} marked as failed.\", task_id);\n",
            "            }\n",
            "            return StatusCode::OK.into_response();\n",
            "        }\n",
            "    }\n",
            "\n",
            "    StatusCode::NOT_FOUND.into_response()\n",
            "}\n"
        ]
        lines[decide_idx:decide_end] = new_decide_func

    with open(path, "w") as f:
        f.writelines(lines)
    print("Successfully updated server.rs with advanced Boss arbitration logic")
else:
    print(f"Failed to find handlers in server.rs")
