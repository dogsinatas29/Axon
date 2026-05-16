
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Fix 1: list_posts -> list_posts_by_thread
# Fix 2: task_id.to_string() -> task_id
# Fix 3: Handle Json mismatch

content = "".join(lines)
content = content.replace("daemon.storage.list_posts(", "daemon.storage.list_posts_by_thread(")
content = content.replace("daemon.storage.get_task(task_id.to_string())", "daemon.storage.get_task(task_id)")

# Rewrite the decide function to be more robust
start_idx = -1
for i, line in enumerate(lines):
    if "async fn submit_semantic_decision" in line:
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
    # Just fix the types inside the existing structure if possible, 
    # but the compiler error 774 suggests there's a WRAPPER calling it.
    pass

# Let's just do a clean rewrite of the whole decision logic to avoid type conflicts
# I'll search for where handle_semantic_decision is defined and calling submit_semantic_decision

with open(path, "w") as f:
    f.write(content)

print("Successfully applied quick fixes to server.rs")
