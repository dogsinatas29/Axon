
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
        while_idx = i + 1
        break

if while_idx != -1:
    # Insert the filter logic
    new_logic = [
        "        let approval_file = path.join(\".axon_approval_pending\");\n",
        "        if approval_file.exists() {\n",
        "            if let Ok(content) = std::fs::read_to_string(\u0026approval_file) {\n",
        "                if let Ok(approval) = serde_json::from_str::<serde_json::Value>(\u0026content) {\n",
        "                    // v0.0.30: [GHOST_FILTER] Hide approved risks\n",
        "                    if approval[\"approved\"].as_bool() != Some(true) {\n",
        "                        if let Some(extra_risks) = approval.get(\"risks\").and_then(|v| v.as_array()) {\n",
        "                            for r_val in extra_risks { risks.push(r_val.clone()); }\n",
        "                        }\n",
        "                    }\n",
        "                }\n",
        "            }\n",
        "        }\n"
    ]
    # Check if there's already something we should replace or just insert
    # Looking at the previous failure, the tool deleted the whole block.
    # We should insert after while_idx.
    lines[while_idx:while_idx] = new_logic
    with open(path, "w") as f:
        f.writelines(lines)
    print("Successfully restored and hardened get_semantic_risks in server.rs")
else:
    print("Could not find while_idx")
