
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Make contract matching more tolerant (basename fallback)
new_lines = []
for line in lines:
    if "if &comp.file_path == target_file {" in line:
        # Check full path OR just the basename
        indent = line[:line.find("if")]
        new_line = f"{indent}let target_name = std::path::Path::new(target_file).file_name().and_then(|n| n.to_str()).unwrap_or(target_file);\n"
        new_line += f"{indent}let comp_name_path = std::path::Path::new(&comp.file_path).file_name().and_then(|n| n.to_str()).unwrap_or(&comp.file_path);\n"
        new_line += f"{indent}if &comp.file_path == target_file || comp_name_path == target_name {{\n"
        new_lines.append(new_line)
    else:
        new_lines.append(line)

with open(path, "w") as f:
    f.writelines(new_lines)
print("Successfully patched server.rs for tolerant contract matching")
