import os
import sys

def strip_markdown(content: str):
    """v0.0.25: Implementation of Step 4."""
    # This is what we WANT to implement in Rust/Python
    lines = content.splitlines()
    clean_lines = [line for line in lines if not line.strip().startswith("```")]
    return "\n".join(clean_lines)

def validate_no_markdown(content: str):
    """POLLUTION_SHIELD: Step 1 Validator."""
    if "```" in content:
        return False
    return True

def test_markdown_contamination():
    contaminated_code = """```rust
fn test() {
    println!("Hello");
}
```"""
    
    print(f"Testing contaminated code:\n{contaminated_code}")
    
    # 1. Validation should FAIL before stripping
    assert validate_no_markdown(contaminated_code) == False, "Validator failed to detect markdown!"
    print("✅ Validator correctly rejected contaminated code.")
    
    # 2. Stripping should work
    clean_code = strip_markdown(contaminated_code)
    print(f"Cleaned code:\n{clean_code}")
    
    # 3. Validation should PASS after stripping
    assert validate_no_markdown(clean_code) == True, "Validator rejected cleaned code!"
    assert "fn test()" in clean_code
    assert "```" not in clean_code
    print("✅ Stripping successfully eliminated markdown.")

if __name__ == "__main__":
    try:
        test_markdown_contamination()
        print("\n🎉 ALL TESTS PASSED!")
    except AssertionError as e:
        print(f"\n❌ TEST FAILED: {e}")
        sys.exit(1)
