
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Find the place where get_semantic_risks was supposed to be
# It's usually after get_queue_api
target_idx = -1
for i, line in enumerate(lines):
    if "async fn get_queue_api" in line:
        # Find the end of that function
        brace_count = 0
        for j in range(i, len(lines)):
            brace_count += lines[j].count('{')
            brace_count -= lines[j].count('}')
            if brace_count == 0 and '{' in "".join(lines[i:j+1]):
                target_idx = j + 1
                break
        break

if target_idx != -1:
    # Check if the header is missing
    if "async fn get_semantic_risks" not in lines[target_idx:target_idx+10]:
        new_header = [
            "\n",
            "async fn get_semantic_risks(State(daemon): State<Arc<Daemon>>) -> Json<serde_json::Value> {\n"
        ]
        lines[target_idx+1:target_idx+1] = new_header # Insert after the newline
        with open(path, "w") as f:
            f.writelines(lines)
        print("Successfully restored function header in server.rs")
    else:
        # If it's there but with _daemon, replace it
        for i in range(target_idx, target_idx+10):
            if "async fn get_semantic_risks" in lines[i]:
                lines[i] = "async fn get_semantic_risks(State(daemon): State<Arc<Daemon>>) -> Json<serde_json::Value> {\n"
                with open(path, "w") as f:
                    f.writelines(lines)
                print("Successfully updated function header in server.rs")
                break
else:
    print("Could not find get_queue_api")
