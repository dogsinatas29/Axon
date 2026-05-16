
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/lib.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Find the start of the rework count check
target_idx = -1
for i, line in enumerate(lines):
    if "if task.rework_count >= 3 {" in line:
        target_idx = i + 1
        break

if target_idx != -1:
    # Find the end of the block (before tokio::time::sleep)
    end_idx = -1
    for i in range(target_idx, len(lines)):
        if "tokio::time::sleep" in lines[i]:
            end_idx = i
            break
    
    if end_idx != -1:
        new_logic = [
            "                        tracing::error!(\"🛑 [FATAL] Task {} failed after 3 reworks.\", task.id);\n",
            "                        let mut failed_task = task.clone();\n",
            "                        failed_task.status = axon_core::TaskStatus::Failed;\n",
            "\n",
            "                        // v0.0.29: AI-driven Failure Diagnosis\n",
            "                        let raw_failures = failures.join(\"\\n---\\n\");\n",
            "                        let diagnosis = self\n",
            "                            .diagnose_failure(task, &raw_failures)\n",
            "                            .await\n",
            "                            .unwrap_or_else(|_| \"진단 실패: 로그 분석 중 오류 발생\".to_string());\n",
            "\n",
            "                        // v0.0.30: [PROVENANCE_HARDENING] Determine rejection source\n",
            "                        let source_label = if raw_failures.contains(\"PHYSICAL_\") {\n",
            "                            \"시스템 게이트 (물리적 정합성 위반)\"\n",
            "                        } else if raw_failures.contains(\"COMPILE_\") || raw_failures.contains(\"error:\") {\n",
            "                            \"컴파일러 (빌드 실패)\"\n",
            "                        } else {\n",
            "                            \"시니어 AI (코드 리뷰 반려)\"\n",
            "                        };\n",
            "\n",
            "                        // v0.0.30: Save Diagnosis & Provenance to Task\n",
            "                        failed_task.error_feedback = Some(format!(\n",
            "                            \"### 🚨 [반려 주체: {}]\\n\\n{}\\n\\n---\\n### 🔍 상세 오류 로그\\n{}\",\n",
            "                            source_label, diagnosis, raw_failures\n",
            "                        ));\n",
            "\n",
            "                        let _ = self.storage.save_task(failed_task.clone()).await;\n",
            "                        let _ = self.storage.update_project_state(&task.project_id, \"ImplGen\", \"failed\").await;\n",
            "\n",
            "                        // v0.0.29: Enrich FATAL message with Senior Audit details\n",
            "                        let senior_report = if let Ok(posts) = self.storage.list_posts_by_thread(&task.id) {\n",
            "                            posts.iter().filter(|p| p.post_type == axon_core::PostType::Review).last()\n",
            "                                .map(|p| format!(\"[SENIOR_REVIEW_AUDIT]: Status='REJECTED'\\n\\\"{}\\\"\", p.content))\n",
            "                                .unwrap_or_else(|| \"[SENIOR_REVIEW_AUDIT]: No review data found.\".to_string())\n",
            "                        } else { \"[SENIOR_REVIEW_AUDIT]: Storage error.\".to_string() };\n",
            "\n",
            "                        self.event_bus.publish(axon_core::Event {\n",
            "                            id: uuid::Uuid::new_v4().to_string(),\n",
            "                            project_id: task.project_id.clone(),\n",
            "                            thread_id: Some(task.id.clone()),\n",
            "                            agent_id: None,\n",
            "                            event_type: axon_core::EventType::AgentAction,\n",
            "                            level: axon_core::EventLevel::Critical,\n",
            "                            source: \"DAEMON\".to_string(),\n",
            "                            content: format!(\"🚨 [{} 반려] {}\\n\\n### 🧠 AI 진단 결과\\n{}\\n\\n---\\n{}\", source_label, task.title, diagnosis, senior_report),\n",
            "                            payload: None,\n",
            "                            timestamp: chrono::Local::now(),\n",
            "                        });\n"
        ]
        lines[target_idx:end_idx] = new_logic
        with open(path, "w") as f:
            f.writelines(lines)
        print("Successfully restored and hardened rejection logic in lib.rs")
    else:
        print("Could not find end_idx")
else:
    print("Could not find target_idx")
