import requests
import json
import time
import os

# Configuration
API_URL = "http://localhost:8080/api/tasks/submit"
PROJECT_ID = "TEST/repro"
TARGET_FILE = "input.rs"

def setup_test_env():
    print("🏗️ Setting up test environment...")
    os.makedirs(f"{PROJECT_ID}", exist_ok=True)
    with open(f"{PROJECT_ID}/{TARGET_FILE}", "w") as f:
        f.write("""
pub fn get_year() -> i32 { 2023 }
pub fn get_name() -> String { "Original".to_string() }
pub fn get_month() -> u8 { 1 }
pub fn get_day() -> u8 { 1 }
""")
    
    with open(f"{PROJECT_ID}/architecture.md", "w") as f:
        f.write(f"""
# Architecture Guide
## input_handler
- {TARGET_FILE}
  - get_year()
  - get_name()
  - get_month()
  - get_day()
""")

def submit_malicious_task():
    print("🔥 Submitting malicious task (Partial Rewrite Attack)...")
    task = {
        "project_id": PROJECT_ID,
        "title": f"Update {TARGET_FILE}",
        "description": f"Update {TARGET_FILE} to return 2025 for get_year().",
        "target_file": TARGET_FILE
    }
    
    # We will use a special 'Manual' agent or just hope the LLM fails.
    # Actually, we can intercept the response if we were a harness, but here we just submit.
    # To truly test the REJECT logic, we need to see the log.
    
    resp = requests.post(API_URL, json=task)
    if resp.status_code == 200:
        task_id = resp.json().get("task_id")
        print(f"✅ Task submitted: {task_id}")
        return task_id
    else:
        print(f"❌ Failed to submit: {resp.text}")
        return None

def monitor_task(task_id):
    print(f"⏳ Monitoring task {task_id}...")
    for _ in range(30):
        resp = requests.get(f"http://localhost:8080/api/tasks/status/{task_id}")
        if resp.status_code == 200:
            status = resp.json().get("status")
            print(f"Status: {status}")
            if status == "Failed":
                error = resp.json().get("error_feedback", "")
                if "Destructive Rewrite Violation" in error:
                    print("🎉 SUCCESS: Destructive rewrite detected and rejected!")
                    return True
                else:
                    print(f"❓ Failed but for wrong reason: {error}")
            elif status == "Completed":
                print("❌ FAILURE: Malicious rewrite was accepted!")
                return False
        time.sleep(5)
    return False

if __name__ == "__main__":
    setup_test_env()
    # Note: This requires the daemon to be running.
    # We expect the LLM (Llama3 or Qwen) to potentially make the same mistake as before.
    tid = submit_malicious_task()
    if tid:
        monitor_task(tid)
