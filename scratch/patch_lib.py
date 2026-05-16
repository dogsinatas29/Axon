
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/lib.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Find the start of execute_junior_task
start_idx = -1
for i, line in enumerate(lines):
    if "fn execute_junior_task" in line:
        start_idx = i
        break

if start_idx == -1:
    print("Could not find execute_junior_task")
    sys.exit(1)

# Find the end of the function (approximately, based on context)
# We want to replace from junior_name definition down to the end of the thought block
replace_start = -1
for i in range(start_idx, len(lines)):
    if "let junior_name =" in line: # wait, check exact string
        pass
    if ".unwrap_or(\"junior-agent-001\");" in lines[i]:
        replace_start = i + 1
        break

replace_end = -1
for i in range(replace_start, len(lines)):
    if "if !parsed_success {" in lines[i]:
        replace_end = i
        break

if replace_start != -1 and replace_end != -1:
    new_code = [
        "        let mut junior_runtime = axon_agent::AgentRuntime::new(\n",
        "            junior_name.to_string(),\n",
        "            axon_core::AgentRole::Junior,\n",
        "            self.junior_model_names[0].clone(),\n",
        "            self.junior_models[0].clone(),\n",
        "        )\n",
        "        .with_project(task.project_id.clone());\n",
        "        junior_runtime.set_locale(&self.locale);\n",
        "\n",
        "        // 3. Execute implementation through axon-agent (Enforcing Stage 1 Policy)\n",
        "        let guide = self.architecture_guide.read().unwrap().clone();\n",
        "        let mut instruction = String::new();\n",
        "        instruction.push_str(\"CRITICAL: If you use sqlite3 functions, you MUST #include <sqlite3.h>. If you use time functions, you MUST #include <time.h>.\\n\");\n",
        "\n",
        "        let post = junior_runtime\n",
        "            .run_implementation_task(\n",
        "                task,\n",
        "                self.event_bus.clone(),\n",
        "                &task.kind,\n",
        "                &instruction,\n",
        "                &guide,\n",
        "                &existing_code,\n",
        "            )\n",
        "            .await?;\n",
        "\n",
        "        let full_code = post.full_code.unwrap_or_default();\n",
        "        let thought_opt = post.thought;\n",
        "        let parsed_success = !full_code.is_empty();\n",
        "        let metrics = post.metrics.unwrap_or(axon_core::RuntimeMetrics {\n",
        "            total_duration: Some(0),\n",
        "            eval_count: Some(0),\n",
        "            eval_duration: Some(0),\n",
        "        });\n",
        "\n",
        "        // v0.0.25: Post the Junior's thought to the Lounge BEFORE parser checks\n",
        "        if let Some(ref thought) = thought_opt {\n"
    ]
    lines[replace_start:replace_end] = new_code
    with open(path, "w") as f:
        f.writelines(lines)
    print("Successfully patched lib.rs")
else:
    print(f"Could not find markers: {replace_start}, {replace_end}")
    # Fallback: find tracing::info!("🍻 [LOUNGE_POST]... and insert before it
    lounge_idx = -1
    for i, line in enumerate(lines):
        if "[LOUNGE_POST]" in line:
            lounge_idx = i
            break
    if lounge_idx != -1:
        # Search back for where junior_runtime was initialized
        # We need to find where junior_name ends
        name_idx = -1
        for i in range(lounge_idx, 0, -1):
            if ".unwrap_or(\"junior-agent-001\");" in lines[i]:
                name_idx = i
                break
        if name_idx != -1:
            new_code = [
                "        let mut junior_runtime = axon_agent::AgentRuntime::new(\n",
                "            junior_name.to_string(),\n",
                "            axon_core::AgentRole::Junior,\n",
                "            self.junior_model_names[0].clone(),\n",
                "            self.junior_models[0].clone(),\n",
                "        )\n",
                "        .with_project(task.project_id.clone());\n",
                "        junior_runtime.set_locale(&self.locale);\n",
                "\n",
                "        // 3. Execute implementation through axon-agent (Enforcing Stage 1 Policy)\n",
                "        let guide = self.architecture_guide.read().unwrap().clone();\n",
                "        let mut instruction = String::new();\n",
                "        instruction.push_str(\"CRITICAL: If you use sqlite3 functions, you MUST #include <sqlite3.h>. If you use time functions, you MUST #include <time.h>.\\n\");\n",
                "\n",
                "        let post = junior_runtime\n",
                "            .run_implementation_task(\n",
                "                task,\n",
                "                self.event_bus.clone(),\n",
                "                &task.kind,\n",
                "                &instruction,\n",
                "                &guide,\n",
                "                &existing_code,\n",
                "            )\n",
                "            .await?;\n",
                "\n",
                "        let full_code = post.full_code.unwrap_or_default();\n",
                "        let thought_opt = post.thought;\n",
                "        let parsed_success = !full_code.is_empty();\n",
                "        let metrics = post.metrics.unwrap_or(axon_core::RuntimeMetrics {\n",
                "            total_duration: Some(0),\n",
                "            eval_count: Some(0),\n",
                "            eval_duration: Some(0),\n",
                "        });\n",
                "\n",
                "        // v0.0.25: Post the Junior's thought to the Lounge BEFORE parser checks\n",
                "        if let Some(ref thought) = thought_opt {\n"
            ]
            lines[name_idx+1:lounge_idx] = new_code
            with open(path, "w") as f:
                f.writelines(lines)
            print("Successfully patched lib.rs via fallback")
        else:
            print("Could not find name_idx")
    else:
        print("Could not find lounge_idx")
