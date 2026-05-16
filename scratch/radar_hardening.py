
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Enhance task gathering: Ensure we look at all tasks regardless of project context
# and add more debug tracing to see what's happening.
new_lines = []
for line in lines:
    if "let tasks = daemon.storage.list_all_tasks().unwrap_or_default();" in line:
        new_lines.append("    tracing::info!(\"🔍 [RADAR] Scanning global task database for semantic risks...\");\n")
        new_lines.append(line)
        new_lines.append("    tracing::info!(\"📊 [RADAR] Total tasks found: {}\", tasks.len());\n")
    elif "if task.rework_count >= 3 {" in line:
        # Also ensure we only show active/unresolved rejections to avoid clutter
        new_lines.append("        if task.rework_count >= 3 && task.status != axon_core::TaskStatus::Completed {\n")
    else:
        new_lines.append(line)

with open(path, "w") as f:
    f.writelines(new_lines)
print("Successfully enhanced task radar and added debug tracing")
