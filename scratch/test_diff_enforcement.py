import sys

def is_single_file_diff(patch_content: str):
    """v0.0.25 Phase 4 Implementation (conceptual for AXON Protocol)."""
    file_blocks = patch_content.count("FILE:")
    return file_blocks == 1

def test_diff_enforcement():
    valid_patch = """===AXON_PATCH_START===
FILE: calculation.rs
ACTION: rewrite
---CODE START---
fn add(a: i32, b: i32) -> i32 { a + b }
---CODE END---
===AXON_PATCH_END==="""

    multi_patch = """===AXON_PATCH_START===
FILE: calculation.rs
ACTION: rewrite
---CODE START---
fn add(a: i32, b: i32) -> i32 { a + b }
---CODE END---
FILE: other.rs
ACTION: rewrite
---CODE START---
fn sub(a: i32, b: i32) -> i32 { a - b }
---CODE END---
===AXON_PATCH_END==="""

    print("Testing single-file patch...")
    assert is_single_file_diff(valid_patch) == True
    print("✅ Correctly accepted single-file patch.")

    print("Testing multi-file patch...")
    assert is_single_file_diff(multi_patch) == False
    print("✅ Correctly rejected multi-file patch.")

if __name__ == "__main__":
    try:
        test_diff_enforcement()
        print("\n🎉 DIFF ENFORCEMENT TEST PASSED!")
    except AssertionError as e:
        print(f"\n❌ TEST FAILED: {e}")
        sys.exit(1)
