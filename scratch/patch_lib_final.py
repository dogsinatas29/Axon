
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/lib.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Find execute_junior_task
start_idx = -1
for i, line in enumerate(lines):
    if "fn execute_junior_task" in line:
        start_idx = i
        break

if start_idx == -1:
    sys.exit(1)

# Find where to start replacement
# Search for .unwrap_or("junior-agent-001");
replace_start = -1
for i in range(start_idx, len(lines)):
    if ".unwrap_or(\"junior-agent-001\");" in lines[i]:
        replace_start = i + 1
        break

# Find where to end replacement
# Search for the diagnose_failure or another safe marker
replace_end = -1
for i in range(replace_start, len(lines)):
    if "fn diagnose_failure" in lines[i]:
        replace_end = i - 1
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
        "        if let Some(ref thought) = thought_opt {\n",
        "            tracing::info!(\"🍻 [LOUNGE_POST] Saving Nogari from {} to lounge...\", junior_name);\n",
        "            let _ = self.storage.save_post(axon_core::Post {\n",
        "                id: uuid::Uuid::new_v4().to_string(),\n",
        "                thread_id: \"lounge\".to_string(),\n",
        "                author_id: junior_name.to_string(),\n",
        "                content: format!(\"**[Task: {}]**\\n{}\", task.title, thought),\n",
        "                post_type: axon_core::PostType::Nogari,\n",
        "                thought: None,\n",
        "                full_code: None,\n",
        "                metrics: None,\n",
        "                created_at: chrono::Local::now(),\n",
        "            }).await;\n",
        "\n",
        "            self.publish_event(axon_core::Event {\n",
        "                id: uuid::Uuid::new_v4().to_string(),\n",
        "                project_id: task.project_id.clone(),\n",
        "                thread_id: Some(\"lounge\".to_string()),\n",
        "                agent_id: task.assigned_worker.clone(),\n",
        "                event_type: axon_core::EventType::MessagePosted,\n",
        "                level: axon_core::EventLevel::Info,\n",
        "                source: format!(\"JUNIOR-{}\", task.assigned_worker.as_deref().unwrap_or(\"unknown\")),\n",
        "                content: format!(\"💬 {}: {}\", task.assigned_worker.as_deref().unwrap_or(\"Junior\"), thought),\n",
        "                payload: None,\n",
        "                timestamp: chrono::Local::now(),\n",
        "            });\n",
        "        }\n",
        "\n",
        "        if !parsed_success {\n",
        "            tracing::error!(\"❌ [PARSER_FAIL] Junior produced a response but it could not be parsed into AXON Patch V2.\");\n",
        "            anyhow::bail!(\"Code Extraction Failed: Junior response did not follow AXON Patch Protocol V2.\");\n",
        "        }\n",
        "\n",
        "        // v0.0.29: [SOVEREIGN_GATE] Physical validation for Header Freeze (C-specific)\n",
        "        if task.kind == \"c\" && task.target_file.as_deref().unwrap_or(\"\").ends_with(\".h\") {\n",
        "            let body_detected = full_code.contains('{')\n",
        "                && (full_code.contains(\"if \") || full_code.contains(\"while \") || full_code.contains(\"return \") || full_code.contains(\"for \"));\n",
        "            if body_detected {\n",
        "                tracing::error!(\"❌ [SOVEREIGN_GATE] Header Freeze Violation detected in {}. Function bodies are forbidden in headers.\", task.target_file.as_deref().unwrap_or(\"unknown\"));\n",
        "                anyhow::bail!(\"HEADER_FREEZE_VIOLATION: Implementation in header forbidden.\");\n",
        "            }\n",
        "        }\n",
        "\n",
        "        let content_with_code = format!(\"### 🧪 [JUNIOR_PROPOSAL]\\n\\n```{} \\n{}\\n```\", task.kind, full_code);\n",
        "        let _ = self.storage.save_post(axon_core::Post {\n",
        "            id: uuid::Uuid::new_v4().to_string(),\n",
        "            thread_id: task.id.clone(),\n",
        "            author_id: junior_name.to_string(),\n",
        "            content: content_with_code,\n",
        "            post_type: axon_core::PostType::Proposal,\n",
        "            thought: thought_opt,\n",
        "            full_code: Some(full_code.clone()),\n",
        "            metrics: Some(metrics.clone()),\n",
        "            created_at: chrono::Local::now(),\n",
        "        }).await;\n",
        "\n",
        "        Ok((full_code, metrics))\n",
        "    }\n"
    ]
    lines[replace_start:replace_end] = new_code
    with open(path, "w") as f:
        f.writelines(lines)
    print("Successfully patched lib.rs with full logic")
