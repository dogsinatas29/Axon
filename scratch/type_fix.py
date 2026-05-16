
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Correct ArchitectureIR to ProjectIR
new_lines = []
for line in lines:
    new_line = line.replace("axon_ir::ArchitectureIR", "axon_ir::ProjectIR")
    new_lines.append(new_line)

with open(path, "w") as f:
    f.writelines(new_lines)
print("Successfully corrected type name in server.rs")
