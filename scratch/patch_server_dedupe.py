
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

# Find while loop start
while_idx = -1
for i in range(start_idx, len(lines)):
    if "while let Some(path) = stack.pop() {" in lines[i]:
        while_idx = i
        break

if while_idx != -1:
    # Insert deduplication setup before the while loop
    new_setup = [
        "    // v0.0.30: [DEDUPLICATION] Get pending projects to hide redundant semantic risks\n",
        "    let pending_projects: std::collections::HashSet<String> = daemon.storage.list_all_threads().unwrap_or_default()\n",
        "        .into_iter()\n",
        "        .filter(|t| t.status == axon_core::ThreadStatus::Draft || t.status == axon_core::ThreadStatus::Working)\n",
        "        .map(|t| t.project_id.clone())\n",
        "        .collect();\n",
        "\n"
    ]
    lines[while_idx:while_idx] = new_setup
    
    # Now find the loop body (re-adjusting while_idx because of insertion)
    new_while_idx = while_idx + len(new_setup) + 1
    
    # Find the block where risks are added (A. Check for Arbitration Risks)
    # Since the previous tool might have deleted or messed it up, we'll replace the block from while_idx down to IR scan
    ir_scan_idx = -1
    for i in range(new_while_idx, len(lines)):
        if "load_project_ir" in lines[i]:
            ir_scan_idx = i
            break
            
    if ir_scan_idx != -1:
        new_loop_body = [
            "        visited_count += 1;\n",
            "        if visited_count > 100 { break; }\n",
            "\n",
            "        let approval_file = path.join(\".axon_approval_pending\");\n",
            "        if approval_file.exists() {\n",
            "            if let Ok(content) = std::fs::read_to_string(\u0026approval_file) {\n",
            "                if let Ok(approval) = serde_json::from_str::<serde_json::Value>(\u0026content) {\n",
            "                    if approval[\"approved\"].as_bool() != Some(true) {\n",
            "                        let project_id = approval[\"project_id\"].as_str().unwrap_or(\"\");\n",
            "                        // v0.0.30: [DEDUPE] Hide if covered by a Pending Thread (Green Card)\n",
            "                        if !pending_projects.contains(project_id) {\n",
            "                            if let Some(extra_risks) = approval.get(\"risks\").and_then(|v| v.as_array()) {\n",
            "                                for r_val in extra_risks { risks.push(r_val.clone()); }\n",
            "                            }\n",
            "                        }\n",
            "                    }\n",
            "                }\n",
            "            }\n",
            "        }\n"
        ]
        # Replace from new_while_idx to ir_scan_idx
        # First, find the closing brace of the previous block or just replace until ir_scan
        lines[new_while_idx:ir_scan_idx] = new_loop_body
        with open(path, "w") as f:
            f.writelines(lines)
        print("Successfully integrated deduplication into get_semantic_risks")
    else:
        print("Could not find ir_scan_idx")
else:
    print("Could not find while_idx")
