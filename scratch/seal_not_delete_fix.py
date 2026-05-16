
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Update handle_semantic_decision to UPDATE the file with "approved": true instead of deleting it
start_idx = -1
for i, line in enumerate(lines):
    if "async fn handle_semantic_decision" in line:
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
        "async fn handle_semantic_decision(\n",
        "    State(daemon): State<Arc<Daemon>>,\n",
        "    Json(payload): Json<serde_json::Value>,\n",
        ") -> impl IntoResponse {\n",
        "    let risk_id = payload[\"risk_id\"].as_str().unwrap_or_default();\n",
        "    let action = payload[\"action\"].as_str().unwrap_or_default();\n",
        "\n",
        "    if risk_id == \"pending_approval\" {\n",
        "        let root = std::env::current_dir().unwrap_or_default();\n",
        "        let mut stack = vec![root];\n",
        "        let mut found = false;\n",
        "        while let Some(path) = stack.pop() {\n",
        "            let approval_file = path.join(\".axon_approval_pending\");\n",
        "            if approval_file.exists() {\n",
        "                if let Ok(content) = std::fs::read_to_string(\u0026approval_file) {\n",
        "                    if let Ok(mut val) = serde_json::from_str::\u003Cserde_json::Value\u003E(\u0026content) {\n",
        "                        val[\"approved\"] = serde_json::json!(true);\n",
        "                        if let Ok(json) = serde_json::to_string_pretty(\u0026val) {\n",
        "                            let _ = std::fs::write(\u0026approval_file, json);\n",
        "                            info!(\"✅ [SOVEREIGN_SEAL] Seal applied to approval file. Factory proceeding...\");\n",
        "                            found = true;\n",
        "                        }\n",
        "                    }\n",
        "                }\n",
        "                if found { break; }\n",
        "            }\n",
        "            if let Ok(entries) = std::fs::read_dir(\u0026path) {\n",
        "                for entry in entries.flatten() {\n",
        "                    if entry.path().is_dir() \u0026\u0026 !entry.file_name().to_string_lossy().starts_with('.') { stack.push(entry.path()); }\n",
        "                }\n",
        "            }\n",
        "        }\n",
        "        if found { return StatusCode::OK; }\n",
        "    }\n",
        "\n",
        "    if risk_id.starts_with(\"rejection_limit_\") {\n",
        "        let task_id = risk_id.replace(\"rejection_limit_\", \"\");\n",
        "        if let Ok(mut task) = daemon.storage.get_task(\u0026task_id) {\n",
        "            match action {\n",
        "                \"SEAL\" =\u003E {\n",
        "                    task.status = crate::TaskStatus::Completed;\n",
        "                    task.rework_count = 0;\n",
        "                    let _ = daemon.storage.update_task(task);\n",
        "                    info!(\"✅ [OVERRIDE_SEAL] Task {} manually sealed by Boss.\", task_id);\n",
        "                }\n",
        "                \"REWORK\" =\u003E {\n",
        "                    task.status = crate::TaskStatus::Pending;\n",
        "                    task.rework_count = 0;\n",
        "                    let _ = daemon.storage.update_task(task);\n",
        "                    info!(\"🔄 [BOSS_REWORK] Task {} queued for re-implementation.\", task_id);\n",
        "                }\n",
        "                \"STOP\" =\u003E {\n",
        "                    task.status = crate::TaskStatus::Failed;\n",
        "                    let _ = daemon.storage.update_task(task);\n",
        "                    info!(\"🛑 [BOSS_STOP] Task {} discarded.\", task_id);\n",
        "                }\n",
        "                _ =\u003E {}\n",
        "            }\n",
        "        }\n",
        "    }\n",
        "\n",
        "    StatusCode::OK\n",
        "}\n"
    ]
    lines[start_idx:end_idx] = new_func

with open(path, "w") as f:
    f.writelines(lines)
print("Successfully patched server.rs to use Seal instead of Delete")
