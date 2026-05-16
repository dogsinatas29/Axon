
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Update get_semantic_risks to support ja_JP for pending_approval
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
        "             let (cause, expected, detected, recommend) = if daemon.locale == \"ko_KR\" {\n",
        "                 (\"새로운 명세가 감지되었습니다. 제조 공정 시작을 위해 보스의 승인이 필요합니다.\", \"승인된 명세 (Authorized Specification)\", \"설계 초안 대기 중 (New Design Draft)\", \"[승인 \u0026 봉인] 버튼을 눌러 공정 시작을 승인하십시오.\")\n",
        "             } else if daemon.locale == \"ja_JP\" {\n",
        "                 (\"新しい仕様が検出されました。製造工程を開始するには社長の承認が必要です。\", \"承認された仕様 (Authorized Specification)\", \"設計ドラフト待機中 (New Design Draft)\", \"[承認 \u0026 封印] ボタンを押して工程の開始を承認してください。\")\n",
        "             } else {\n",
        "                 (\"New specification detected. Boss approval required to start manufacturing.\", \"Authorized Specification\", \"New Design Draft awaiting approval\", \"Click [OVERRIDE \u0026 SEAL] to approve.\")\n",
        "             };\n",
        "\n",
        "             risks.push(serde_json::json!({\n",
        "                 \"risk_id\": \"pending_approval\",\n",
        "                 \"kind\": \"Bootstrap\",\n",
        "                 \"level\": \"Critical\",\n",
        "                 \"target\": \"Factory Gateway\",\n",
        "                 \"failed_stage\": \"SpecAnalysis\",\n",
        "                 \"actor\": if daemon.locale == \"ko_KR\" { \"Sovereign Gatekeeper\" } else if daemon.locale == \"ja_JP\" { \"統治官 (Sovereign Gatekeeper)\" } else { \"Sovereign Gatekeeper\" },\n",
        "                 \"cause\": cause,\n",
        "                 \"expected\": expected,\n",
        "                 \"detected\": detected,\n",
        "                 \"recommendation\": recommend,\n",
        "                 \"component\": \"GATEWAY\",\n",
        "             }));\n",
        "        }\n",
        "        // ... rest of the logic remains same for IR extraction\n",
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
        "    // Add tasks logic (omitted here but should be preserved in real file)\n",
        "    Json(serde_json::json!({ \"risks\": risks, \"locale\": daemon.locale }))\n",
        "}\n"
    ]
    # In reality I should preserve the task logic. I'll do a more careful replace or rewrite.
    # For now, let's just use the previous full function content but with ja_JP added.
    pass

# I'll rewrite the entire get_semantic_risks function carefully in the next step.
print("Backend ja_JP mapping prepared.")
