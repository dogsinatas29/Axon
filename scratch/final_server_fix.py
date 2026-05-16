
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Clean up the end of the file and unify types
new_lines = []
skip = False
for line in lines:
    if "async fn respond_approval" in line:
        new_lines.append("async fn respond_approval(State(daemon): State<Arc<Daemon>>, Json(decision): Json<serde_json::Value>) -> impl IntoResponse {\n")
        new_lines.append("    submit_semantic_decision(State(daemon), Json(decision)).await\n")
        new_lines.append("}\n")
        skip = True
    elif skip:
        if line.strip() == "}" or line.startswith("async fn") or line.startswith("pub fn"):
            skip = False
            if line.strip() == "}": continue
    if not skip:
        new_lines.append(line)

content = "".join(new_lines)
content = content.replace("daemon.storage.list_posts(", "daemon.storage.list_posts_by_thread(")
content = content.replace("daemon.storage.get_task(task_id.to_string())", "daemon.storage.get_task(task_id)")

with open(path, "w") as f:
    f.write(content)

print("Successfully unified server.rs types and fixed storage calls")
