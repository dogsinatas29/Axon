
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Find submit_semantic_decision
start_idx = -1
for i, line in enumerate(lines):
    if "async fn submit_semantic_decision" in line:
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
    # Inject special case for "pending_approval" (Bootstrap Approval)
    clean_func = [
        "async fn submit_semantic_decision(State(daemon): State<Arc<Daemon>>, Json(decision): Json<axon_core::validator::SemanticDecision>) -> impl IntoResponse {\n",
        "    let root_path = std::env::current_dir().unwrap_or_default();\n",
        "    let mut stack = vec![root_path];\n",
        "    let mut visited_count = 0;\n",
        "    while let Some(path) = stack.pop() {\n",
        "        visited_count += 1;\n",
        "        if visited_count > 100 { break; }\n",
        "        let approval_file = path.join(\".axon_approval_pending\");\n",
        "        if approval_file.exists() {\n",
        "            if let Ok(content) = std::fs::read_to_string(\u0026approval_file) {\n",
        "                if let Ok(mut approval) = serde_json::from_str::<serde_json::Value>(\u0026content) {\n",
        "                    // v0.0.30: [SPECIAL_APPROVAL_BYPASS] Allow \"pending_approval\" ID for bootstrap gating\n",
        "                    let is_bootstrap_approval = decision.risk_id == \"pending_approval\";\n",
        "                    let match_found = if is_bootstrap_approval {\n",
        "                        true\n",
        "                    } else if let Some(risks) = approval.get(\"risks\").and_then(|v| v.as_array()) {\n",
        "                        risks.iter().any(|r| r[\"id\"].as_str() == Some(\u0026decision.risk_id))\n",
        "                    } else { false };\n",
        "\n",
        "                    if match_found {\n",
        "                        daemon.publish_event(axon_core::Event {\n",
        "                            id: uuid::Uuid::new_v4().to_string(),\n",
        "                            project_id: \"system\".to_string(),\n",
        "                            thread_id: None, agent_id: None,\n",
        "                            event_type: axon_core::EventType::Signal,\n",
        "                            author_id: \"SYSTEM\".to_string(),\n",
        "                            content: format!(\"✅ **[Governance]** Boss finalized decision for project ID: **{}**.\", decision.risk_id),\n",
        "                            created_at: chrono::Local::now(),\n",
        "                            metadata: std::collections::HashMap::new(),\n",
        "                        }).await;\n",
        "                        \n",
        "                        let _ = daemon.storage.save_post(axon_core::Post {\n",
        "                            id: uuid::Uuid::new_v4().to_string(),\n",
        "                            thread_id: \"lounge\".to_string(),\n",
        "                            author_id: \"SYSTEM\".to_string(),\n",
        "                            content: format!(\"📢 **[Governance Alert]** Boss finalized arbitration on **{}**. Action: **{:?}**.\", decision.risk_id, decision.action),\n",
        "                            post_type: axon_core::PostType::Nogari,\n",
        "                            thought: None, full_code: None, metrics: None, created_at: chrono::Local::now(),\n",
        "                        }).await;\n",
        "                        \n",
        "                        approval[\"approved\"] = serde_json::json!(decision.action == axon_core::validator::DecisionAction::Approve || decision.action == axon_core::validator::DecisionAction::Seal);\n",
        "                        approval[\"opinion\"] = serde_json::json!(decision.comment);\n",
        "                        if let Ok(new_content) = serde_json::to_string_pretty(\u0026approval) {\n",
        "                            let _ = std::fs::write(\u0026approval_file, new_content);\n",
        "                            return StatusCode::OK.into_response();\n",
        "                        }\n",
        "                    }\n",
        "                }\n",
        "            }\n",
        "        }\n",
        "        if let Ok(entries) = std::fs::read_dir(\u0026path) {\n",
        "            for entry in entries.flatten() {\n",
        "                if entry.path().is_dir() \u0026\u0026 !entry.file_name().to_string_lossy().starts_with('.') { stack.push(entry.path()); }\n",
        "            }\n",
        "        }\n",
        "    }\n",
        "    StatusCode::NOT_FOUND.into_response()\n",
        "}\n"
    ]
    lines[start_idx:end_idx] = clean_func
    with open(path, "w") as f:
        f.writelines(lines)
    print("Successfully implemented bootstrap approval bypass in server.rs")
else:
    print(f"Could not find start/end: {start_idx}, {end_idx}")
