
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    content = f.read()

# Replace interfaces with functions
content = content.replace(".interfaces.iter()", ".functions.iter()")

with open(path, "w") as f:
    f.write(content)

print("Successfully fixed field name in server.rs Smart Diagnosis")
