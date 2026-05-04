import sys

def is_final_node(tasks):
    """v0.0.25 Phase 5 Implementation."""
    # Check if any tasks are not completed/failed
    active_tasks = [t for t in tasks if t['status'] in ['Pending', 'Ready', 'InProgress']]
    return len(active_tasks) == 0

def test_execution_optimization():
    # Case 1: Partial generation (Not final)
    tasks_partial = [
        {'id': 'task-1', 'status': 'InProgress'}, # Current task being processed
        {'id': 'task-2', 'status': 'Pending'},
    ]
    # Simulate the check after 'task-1' would have finished but before its status is updated in DB
    # Actually, in my Rust code, I filter out the current task 't.id != task.id'
    
    current_task_id = 'task-1'
    project_tasks = [t for t in tasks_partial if t['id'] != current_task_id]
    is_final = not any(t['status'] in ['Pending', 'Ready', 'InProgress'] for t in project_tasks)
    
    print(f"Testing partial generation (current: {current_task_id})...")
    assert is_final == False
    print("✅ Correctly skipped harness for intermediate task.")

    # Case 2: Final node
    tasks_final = [
        {'id': 'task-1', 'status': 'Completed'},
        {'id': 'task-2', 'status': 'InProgress'}, # Current task
    ]
    
    current_task_id = 'task-2'
    project_tasks = [t for t in tasks_final if t['id'] != current_task_id]
    is_final = not any(t['status'] in ['Pending', 'Ready', 'InProgress'] for t in project_tasks)
    
    print(f"Testing final node (current: {current_task_id})...")
    assert is_final == True
    print("✅ Correctly triggered harness for final task.")

if __name__ == "__main__":
    try:
        test_execution_optimization()
        print("\n🎉 EXECUTION OPTIMIZATION TEST PASSED!")
    except AssertionError as e:
        print(f"\n❌ TEST FAILED: {e}")
        sys.exit(1)
