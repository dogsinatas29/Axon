
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/lib.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Find lock_in_architecture
target_idx = -1
for i, line in enumerate(lines):
    if "fn lock_in_architecture" in line:
        # Find the end of this function
        brace_count = 0
        for j in range(i, len(lines)):
            brace_count += lines[j].count('{')
            brace_count -= lines[j].count('}')
            if brace_count == 0 and '{' in "".join(lines[i:j+1]):
                target_idx = j + 1
                break
        break

if target_idx != -1:
    new_method = [
        "\n",
        "    /// v0.0.30: [UNIFIED_GOVERNANCE] Automatically seals semantic risks in a project\n",
        "    pub async fn seal_semantic_risks(&self, project_id: &str) -> anyhow::Result<()> {\n",
        "        let root_path = std::env::current_dir().unwrap_or_default();\n",
        "        let mut stack = vec![root_path];\n",
        "        while let Some(path) = stack.pop() {\n",
        "            let approval_file = path.join(\".axon_approval_pending\");\n",
        "            if approval_file.exists() {\n",
        "                if let Ok(content) = std::fs::read_to_string(&approval_file) {\n",
        "                    if let Ok(mut approval) = serde_json::from_str::<serde_json::Value>(&content) {\n",
        "                        if approval[\"project_id\"].as_str() == Some(project_id) {\n",
        "                            tracing::info!(\"⚖️ [UNIFIED_SEAL] Sealing risk in project: {}\", project_id);\n",
        "                            approval[\"approved\"] = serde_json::json!(true);\n",
        "                            approval[\"opinion\"] = serde_json::json!(\"보스가 스레드 승인을 통해 통합 관리함\");\n",
        "                            let _ = std::fs::write(&approval_file, serde_json::to_string_pretty(&approval)?);\n",
        "                        }\n",
        "                    }\n",
        "                }\n",
        "            }\n",
        "            if let Ok(entries) = std::fs::read_dir(\u0026path) {\n",
        "                for entry in entries.flatten() {\n",
        "                    if entry.path().is_dir() \u0026\u0026 !entry.file_name().to_string_lossy().starts_with('.') {\n",
        "                        stack.push(entry.path());\n",
        "                    }\n",
        "                }\n",
        "            }\n",
        "        }\n",
        "        Ok(())\n",
        "    }\n"
    ]
    lines[target_idx:target_idx] = new_method
    with open(path, "w") as f:
        f.writelines(lines)
    print("Successfully added seal_semantic_risks to lib.rs")
else:
    print("Could not find lock_in_architecture")
