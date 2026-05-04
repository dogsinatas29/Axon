import sys

def detect_scope_violation(content: str):
    forbidden_patterns = ["mod ", "use crate::", "../", "File::open", "File::create"]
    for p in forbidden_patterns:
        if p in content:
            return True, p
    return False, None

def test_scope_violation():
    violations = [
        ("mod my_module;", "mod "),
        ("use crate::something;", "use crate::"),
        ('let data = std::fs::read_to_string("../secret.txt");', "../"),
        ("let f = File::open(\"test.txt\");", "File::open"),
    ]
    
    for code, expected_pattern in violations:
        print(f"Testing code:\n{code}")
        is_violation, pattern = detect_scope_violation(code)
        assert is_violation == True, f"Failed to detect violation for: {code}"
        assert pattern == expected_pattern, f"Detected wrong pattern. Expected {expected_pattern}, got {pattern}"
        print(f"✅ Correctly detected violation: {pattern}")

    clean_code = "pub fn add(a: i32, b: i32) -> i32 { a + b }"
    is_violation, _ = detect_scope_violation(clean_code)
    assert is_violation == False, "Incorrectly flagged clean code as violation!"
    print("✅ Clean code passed.")

if __name__ == "__main__":
    try:
        test_scope_violation()
        print("\n🎉 ALL SCOPE CONTROL TESTS PASSED!")
    except AssertionError as e:
        print(f"\n❌ TEST FAILED: {e}")
        sys.exit(1)
