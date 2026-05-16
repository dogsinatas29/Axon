
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Fix the Tuple access: c -> c.1
content = "".join(lines)
content = content.replace("c.name.contains(target)", "c.1.name.contains(target)")
content = content.replace("target.contains(&c.name)", "target.contains(&c.1.name)")
content = content.replace("comp.interfaces.iter()", "comp.1.interfaces.iter()")

with open(path, "w") as f:
    f.write(content)

print("Successfully fixed Tuple access in server.rs Smart Diagnosis")
