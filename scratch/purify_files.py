import os
import re

def clean_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 1. Try AXON Patch Protocol extraction
    match = re.search(r'---CODE START---\n(.*?)\n---CODE END---', content, re.DOTALL)
    if match:
        pure_code = match.group(1)
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(pure_code.strip())
        print(f"✅ Purified (AXON Patch): {filepath}")
        return

    # 2. Try Markdown block extraction
    match = re.search(r'```(?:\w+)?\n(.*?)\n```', content, re.DOTALL)
    if match:
        pure_code = match.group(1)
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(pure_code.strip())
        print(f"✅ Purified (Markdown): {filepath}")
        return
    
    # 3. Simple line-based fallback for AXON markers
    if "===AXON_PATCH_START===" in content:
        lines = content.split('\n')
        pure_lines = []
        in_code = False
        for line in lines:
            if "---CODE START---" in line:
                in_code = True
                continue
            if "---CODE END---" in line:
                in_code = False
                continue
            if in_code:
                pure_lines.append(line)
        
        if pure_lines:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write('\n'.join(pure_lines).strip())
            print(f"✅ Purified (Line-based): {filepath}")
        else:
            print(f"⚠️ Failed to purify: {filepath}")

def main():
    base_dir = "./TEST2/spec/"
    if not os.path.exists(base_dir):
        print(f"Dir {base_dir} not found.")
        return

    for root, dirs, files in os.walk(base_dir):
        for file in files:
            if file.endswith(('.c', '.h', '.py', '.rs', '.js', '.ts', '.tsx', '.css')):
                clean_file(os.path.join(root, file))

if __name__ == "__main__":
    main()
