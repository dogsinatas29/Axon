import os
import shutil
import tempfile

def write_target_only(target_path, file_map):
    """v0.0.25 Phase 3 Implementation."""
    os.makedirs(os.path.dirname(target_path), exist_ok=True)
    
    written_files = []
    for fname, content in file_map.items():
        # Physical Write Gate logic
        if os.path.basename(fname) == os.path.basename(target_path):
            with open(target_path, "w") as f:
                f.write(content)
            written_files.append(fname)
        else:
            print(f"🛡️ [WRITE_GATE] Blocked write to {fname}")
            
    return written_files

def test_write_gate():
    with tempfile.TemporaryDirectory() as tmp_dir:
        target_path = os.path.join(tmp_dir, "allowed.rs")
        forbidden_path = os.path.join(tmp_dir, "forbidden.rs")
        
        file_map = {
            "allowed.rs": "fn allowed() {}",
            "forbidden.rs": "fn forbidden() {}"
        }
        
        print(f"Attempting to write files to {tmp_dir}")
        written = write_target_only(target_path, file_map)
        
        # 1. Target file should exist
        assert os.path.exists(target_path)
        assert "allowed.rs" in written
        
        # 2. Forbidden file should NOT exist
        assert not os.path.exists(forbidden_path)
        assert "forbidden.rs" not in written
        
        print("✅ Write gate successfully blocked unauthorized files.")

if __name__ == "__main__":
    try:
        test_write_gate()
        print("\n🎉 WRITE GATE TEST PASSED!")
    except AssertionError as e:
        print(f"\n❌ TEST FAILED: {e}")
        import sys
        sys.exit(1)
