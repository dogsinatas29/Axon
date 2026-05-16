
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    content = f.read()

# Fix the Function Tuple access: i -> i.1
content = content.replace("map(|i| i.name.clone())", "map(|i| i.1.name.clone())")

with open(path, "w") as f:
    f.write(content)

print("Successfully fixed Function Tuple access in server.rs Smart Diagnosis")
