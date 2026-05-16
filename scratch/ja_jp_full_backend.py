
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Full rewrite of get_semantic_risks with ja_JP and locale propagation
start_idx = -1
for i, line in enumerate(lines):
    if "async fn get_semantic_risks" in line:
        start_idx = i
        break

end_idx = -1
brace_count = 0
for i in range(start_idx, len(lines)):
    brace_count += lines[i].count('{')
    brace_count -= lines[i].count('}')
    if brace_count == 0 and '{' in "".join(lines[start_idx:i+1]):
        end_idx = i + 1
        break

if start_idx != -1 and end_idx != -1:
    new_func = [
        "async fn get_semantic_risks(State(daemon): State<Arc<Daemon>>) -> Json<serde_json::Value> {\n",
        "    let mut risks: Vec<serde_json::Value> = Vec::new();\n",
        "    let root_path = std::env::current_dir().unwrap_or_default();\n",
        "    let mut stack = vec![root_path];\n",
        "    let mut visited_count = 0;\n",
        "\n",
        "    while let Some(path) = stack.pop() {\n",
        "        visited_count += 1;\n",
        "        if visited_count > 100 { break; }\n",
        "        let approval_file = path.join(\".axon_approval_pending\");\n",
        "        if approval_file.exists() {\n",
        "             let (actor, cause, expected, detected, recommend) = if daemon.locale == \"ko_KR\" {\n",
        "                 (\"Sovereign Gatekeeper\", \"새로운 명세가 감지되었습니다. 제조 공정 시작을 위해 보스의 승인이 필요합니다.\", \"승인된 명세 (Authorized Specification)\", \"설계 초안 대기 중 (New Design Draft)\", \"[승인 \u0026 봉인] 버튼을 눌러 공정 시작을 승인하십시오.\")\n",
        "             } else if daemon.locale == \"ja_JP\" {\n",
        "                 (\"統治官 (Sovereign Gatekeeper)\", \"新しい仕様が検出されました。製造工程を開始するには社長の承認が必要です。\", \"承認された仕様 (Authorized Specification)\", \"設計ドラ프트待機中 (New Design Draft)\", \"[承認 \u0026 封印] ボタンを押して工程の開始を承認してください。\")\n",
        "             } else {\n",
        "                 (\"Sovereign Gatekeeper\", \"New specification detected. Boss approval required to start manufacturing.\", \"Authorized Specification\", \"New Design Draft awaiting approval\", \"Click [OVERRIDE \u0026 SEAL] to approve.\")\n",
        "             };\n",
        "\n",
        "             risks.push(serde_json::json!({\n",
        "                 \"risk_id\": \"pending_approval\",\n",
        "                 \"kind\": \"Bootstrap\",\n",
        "                 \"level\": \"Critical\",\n",
        "                 \"target\": \"Factory Gateway\",\n",
        "                 \"failed_stage\": \"SpecAnalysis\",\n",
        "                 \"actor\": actor,\n",
        "                 \"cause\": cause,\n",
        "                 \"expected\": expected,\n",
        "                 \"detected\": detected,\n",
        "                 \"recommendation\": recommend,\n",
        "                 \"component\": \"GATEWAY\",\n",
        "             }));\n",
        "        }\n",
        "        if let Some(ir) = crate::intelligence::decision::load_project_ir(\u0026path.to_string_lossy()) {\n",
        "            let extractor = crate::intelligence::semantic_debugger::SemanticRiskExtractor::new(\u0026path.to_string_lossy());\n",
        "            let extracted = extractor.extract_risks(\u0026ir).await;\n",
        "            for risk in extracted.risks { risks.push(serde_json::to_value(risk).unwrap()); }\n",
        "        }\n",
        "        if let Ok(entries) = std::fs::read_dir(\u0026path) {\n",
        "            for entry in entries.flatten() {\n",
        "                if entry.path().is_dir() \u0026\u0026 !entry.file_name().to_string_lossy().starts_with('.') \u0026\u0026 entry.file_name() != \"target\" { stack.push(entry.path()); }\n",
        "            }\n",
        "        }\n",
        "    }\n",
        "\n",
        "    let tasks = daemon.storage.list_all_tasks().unwrap_or_default();\n",
        "    for task in tasks {\n",
        "        if task.rework_count >= 3 {\n",
        "            let posts = daemon.storage.list_posts_by_thread(\u0026task.id).unwrap_or_default();\n",
        "            let error_post = posts.iter().rev().find(|p| p.author_id != \"BOSS\" \u0026\u0026 (p.content.to_lowercase().contains(\"error\") || p.content.to_lowercase().contains(\"reject\") || p.content.to_lowercase().contains(\"fail\")));\n",
        "            let raw_log = error_post.map(|p| p.content.clone()).unwrap_or_else(|| task.error_feedback.clone().unwrap_or_else(|| \"Unknown Failure\".to_string()));\n",
        "            let last_code = posts.iter().rev().find(|p| p.full_code.is_some()).and_then(|p| p.full_code.clone());\n",
        "            \n",
        "            let mut actor = if daemon.locale == \"ja_JP\" { \"契約検証官\" } else { \"Contract Verifier\" };\n",
        "            let mut failed_stage = if daemon.locale == \"ja_JP\" { \"契約検証\" } else { \"Contract Verification\" };\n",
        "            if raw_log.contains(\"error:\") || raw_log.contains(\"cmake\") {\n",
        "                actor = if daemon.locale == \"ja_JP\" { \"コンパイラ (Clang/GCC)\" } else { \"Compiler (Clang/GCC)\" };\n",
        "                failed_stage = if daemon.locale == \"ja_JP\" { \"ビルド/リンク\" } else { \"Build/Linking\" };\n",
        "            } else if raw_log.contains(\"SENIOR_REJECT\") || raw_log.contains(\"Review\") {\n",
        "                actor = if daemon.locale == \"ja_JP\" { \"シニアAI監査役\" } else { \"Senior AI Auditor\" };\n",
        "                failed_stage = if daemon.locale == \"ja_JP\" { \"意味論的レビュー\" } else { \"Semantic Review\" };\n",
        "            }\n",
        "\n",
        "            let mut target_line = -1;\n",
        "            if let Some(caps) = regex::Regex::new(r\"[:\\s](\\d+)[:\\s]\").ok().and_then(|re| re.captures(\u0026raw_log)) {\n",
        "                target_line = caps.get(1).and_then(|m| m.as_str().parse::\u003Ci32\u003E().ok()).unwrap_or(-1);\n",
        "            }\n",
        "            risks.push(serde_json::json!({\n",
        "                \"risk_id\": format!(\"rejection_limit_{}\", task.id),\n",
        "                \"kind\": \"ImplementationFail\",\n",
        "                \"level\": \"Critical\",\n",
        "                \"target\": task.title,\n",
        "                \"actor\": actor,\n",
        "                \"failed_stage\": failed_stage,\n",
        "                \"cause\": raw_log,\n",
        "                \"target_line\": target_line,\n",
        "                \"component\": task.target_file.clone().unwrap_or_else(|| \"unknown\".to_string()),\n",
        "                \"full_code\": last_code,\n",
        "                \"task_id\": task.id,\n",
        "            }));\n",
        "        }\n",
        "    }\n",
        "    Json(serde_json::json!({ \"risks\": risks, \"locale\": daemon.locale }))\n",
        "}\n"
    ]
    lines[start_idx:end_idx] = new_func

with open(path, "w") as f:
    f.writelines(lines)
print("Successfully patched server.rs for ja_JP support and locale propagation")
