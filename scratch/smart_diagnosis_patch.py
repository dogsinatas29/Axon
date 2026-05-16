
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Update get_semantic_risks to perform "Smart Diagnosis"
# We'll inject a helper to find the expected interface from IR

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
        "             risks.push(serde_json::json!({\n",
        "                 \"risk_id\": \"pending_approval\",\n",
        "                 \"kind\": \"Bootstrap\",\n",
        "                 \"level\": \"Critical\",\n",
        "                 \"target\": \"Factory Gateway\",\n",
        "                 \"message\": \"Bootstrap approval required to begin manufacturing.\",\n",
        "                 \"cause\": \"Bootstrap protocol mandates human arbitration for new project specs.\",\n",
        "                 \"impact\": \"Factory remains idle until this design is authorized.\",\n",
        "                 \"recommendation\": \"Press [SEAL] to authorize the spec.\",\n",
        "                 \"component\": \"BOOTSTRAP\",\n",
        "             }));\n",
        "        }\n",
        "        if let Some(ir) = crate::intelligence::decision::load_project_ir(\u0026path.to_string_lossy()) {\n",
        "            let extractor = crate::intelligence::semantic_debugger::SemanticRiskExtractor::new(\u0026path.to_string_lossy());\n",
        "            let extracted = extractor.extract_risks(\u0026ir).await;\n",
        "            for risk in extracted.risks {\n",
        "                let mut r_val = serde_json::to_value(risk).unwrap_or_default();\n",
        "                if let Some(obj) = r_val.as_object_mut() {\n",
        "                    let rid = obj.get(\"risk_id\").and_then(|v| v.as_str()).unwrap_or(\"unknown_risk\");\n",
        "                    obj.insert(\"risk_id\".to_string(), serde_json::Value::String(rid.to_string()));\n",
        "                    risks.push(serde_json::to_value(obj).unwrap());\n",
        "                }\n",
        "            }\n",
        "        }\n",
        "        if let Ok(entries) = std::fs::read_dir(\u0026path) {\n",
        "            for entry in entries.flatten() {\n",
        "                if entry.path().is_dir() \u0026\u0026 !entry.file_name().to_string_lossy().starts_with('.') \u0026\u0026 entry.file_name() != \"target\" { stack.push(entry.path()); }\n",
        "            }\n",
        "        }\n",
        "    }\n",
        "\n",
        "    // v0.0.30: [SMART_DIAGNOSIS] Enrich quarantined tasks with IR interface mapping\n",
        "    let tasks = daemon.storage.list_all_tasks().unwrap_or_default();\n",
        "    for task in tasks {\n",
        "        if task.rework_count >= 3 {\n",
        "            let posts = daemon.storage.list_posts_by_thread(\u0026task.id).unwrap_or_default();\n",
        "            let last_code = posts.iter().rev().find(|p| p.full_code.is_some()).and_then(|p| p.full_code.clone());\n",
        "            \n",
        "            let mut tactical_guide = \"AI가 코드를 분석 중입니다...\".to_string();\n",
        "            if let Some(target) = \u0026task.target_file {\n",
        "                // Try to find matching component in IR to generate guide\n",
        "                let root = std::env::current_dir().unwrap_or_default();\n",
        "                if let Some(ir) = crate::intelligence::decision::load_project_ir(\u0026root.to_string_lossy()) {\n",
        "                    if let Some(comp) = ir.components.iter().find(|c| c.name.contains(target) || target.contains(\u0026c.name)) {\n",
        "                        let expected_funcs: Vec\u003CString\u003E = comp.interfaces.iter().map(|i| i.name.clone()).collect();\n",
        "                        let code = last_code.clone().unwrap_or_default();\n",
        "                        let mut missing = Vec::new();\n",
        "                        for f in \u0026expected_funcs {\n",
        "                            if !code.contains(f) { missing.push(f.clone()); }\n",
        "                        }\n",
        "                        if !missing.is_empty() {\n",
        "                            tactical_guide = format!(\"⚠️ 위반 사항: 설계된 함수 [{}]가 코드에서 발견되지 않습니다. 대신 다른 명칭을 사용했는지 확인하십시오.\", missing.join(\", \"));\n",
        "                        } else {\n",
        "                            tactical_guide = \"물리적 구조는 일치하나, 내부 로직의 세만틱 위반(시니어 반려)이 의심됩니다.\".to_string();\n",
        "                        }\n",
        "                    }\n",
        "                }\n",
        "            }\n",
        "\n",
        "            risks.push(serde_json::json!({\n",
        "                \"risk_id\": format!(\"rejection_limit_{}\", task.id),\n",
        "                \"kind\": \"ImplementationFail\",\n",
        "                \"level\": \"Critical\",\n",
        "                \"target\": task.title,\n",
        "                \"message\": format!(\"Task '{}' quarantined (3 rejections)\", task.title),\n",
        "                \"cause\": task.error_feedback.clone().unwrap_or_else(|| \"Validator or Build Failure\".to_string()),\n",
        "                \"tactical_guide\": tactical_guide,\n",
        "                \"recommendation\": \"Review the Tactical Guide below and issue a Boss Correction.\",\n",
        "                \"component\": task.target_file.clone().unwrap_or_else(|| \"unknown\".to_string()),\n",
        "                \"full_code\": last_code,\n",
        "                \"task_id\": task.id,\n",
        "            }));\n",
        "        }\n",
        "    }\n",
        "    Json(serde_json::json!({ \"risks\": risks, \"decisions\": [], \"is_sealed\": false }))\n",
        "}\n"
    ]
    lines[start_idx:end_idx] = new_func

with open(path, "w") as f:
    f.writelines(lines)
print("Successfully patched server.rs with Smart Diagnosis logic")
