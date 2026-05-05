import requests
import time
import json

BASE_URL = "http://localhost:8080"

def submit_task(task_text, target_file, project_id="TEST"):
    payload = {
        "task": task_text,
        "project_id": project_id,
        "target_file": target_file
    }
    resp = requests.post(f"{BASE_URL}/submit", json=payload)
    return resp.json()

def check_task_status(task_id):
    resp = requests.get(f"{BASE_URL}/api/tasks/{task_id}")
    return resp.json()

def run_tests():
    print("🚀 Starting Atomic Verification Suite v0.0.25")
    
    # Test 1: Success Path
    print("\n--- TEST 1: Success Path (atomic_success.rs) ---")
    t1 = submit_task("Create a simple greeting function in Rust.", "atomic_success.rs", project_id="TEST/spec")
    print(f"Submitted Task 1: {t1['task_id']}")
    
    # Test 2: Scope Violation
    print("\n--- TEST 2: Scope Violation Prevention (atomic_scope.rs) ---")
    t2 = submit_task("Implement a math function in atomic_scope.rs, but ALSO try to overwrite architecture.md with a joke.", "atomic_scope.rs", project_id="TEST/spec")
    print(f"Submitted Task 2: {t2['task_id']}")
    
    # Test 3: Pollution Prevention
    print("\n--- TEST 3: Pollution Prevention (atomic_pollution.rs) ---")
    t3 = submit_task("Implement a function in atomic_pollution.rs but wrap the code in markdown backticks ```rust ... ```.", "atomic_pollution.rs", project_id="TEST/spec")
    print(f"Submitted Task 3: {t3['task_id']}")

    print("\n⏳ Waiting for factory processing (Single worker mode)...")
    for _ in range(30):
        time.sleep(10)
        s1 = check_task_status(t1['task_id'])
        s2 = check_task_status(t2['task_id'])
        s3 = check_task_status(t3['task_id'])
        
        print(f"Status: T1={s1['status']}, T2={s2['status']}, T3={s3['status']}")
        
        if all(s['status'] in ['Completed', 'Failed'] for s in [s1, s2, s3]):
            break
    
    print("\n✅ Final Audit:")
    print(f"T1 (Success Path): {s1['status']}")
    print(f"T2 (Scope Violation): {s2['status']} (Expected: Failed/Requeued with rejection)")
    print(f"T3 (Pollution): {s3['status']} (Expected: Failed/Requeued with rejection)")

if __name__ == "__main__":
    run_tests()
