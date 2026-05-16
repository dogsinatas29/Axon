
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Correct field from 'contracts' to 'functions'
new_lines = []
for line in lines:
    new_line = line.replace("comp.contracts.iter().map(|c| c.name.clone()).collect::<Vec<_>>().join(\", \")", 
                            "comp.functions.keys().cloned().collect::<Vec<_>>().join(\", \")")
    new_lines.append(new_line)

with open(path, "w") as f:
    f.writelines(new_lines)
print("Successfully corrected field name in server.rs")
