use crate::spec_ir::{
    SemanticSpecIR, LogicalNode, InterfaceContract, StateTransition, TransitionKind, RetryLoop,
    ArchitecturalInvariant, DependencyConstraint,
};
use crate::schema::Language;
use pulldown_cmark::{Parser, Event, Tag, TagEnd};

fn clean_file_path(raw: &str) -> String {
    let clean = raw.split('/').next().unwrap_or(raw);
    clean.trim_matches(|c| c == '`' || c == '*' || c == '_' || c == '(' || c == ')').to_string()
}

pub fn parse_spec(markdown: &str) -> SemanticSpecIR {
    let mut ir = SemanticSpecIR::default();
    let parser = Parser::new(markdown);

    let mut current_section = String::new();
    let mut in_table = false;
    let mut table_headers: Vec<String> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut in_code_block = false;
    let mut current_code = String::new();
    let mut in_list = false;
    let mut current_list_item = String::new();
    let mut in_paragraph = false;
    let mut current_paragraph = String::new();

    let mut _active_component_file = String::new();
    let mut active_component_node_id = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                current_section.clear();
            }
            Event::Text(text) => {
                let t = text.into_string();
                if current_section.is_empty() {
                    current_section.push_str(&t);
                    if current_section.to_lowercase().contains("rust") {
                        ir.language = Language::Rust;
                    } else if current_section.to_lowercase().contains("c++") || current_section.to_lowercase().contains("cpp") {
                        ir.language = Language::Cpp;
                    } else if current_section.to_lowercase().contains("python") {
                        ir.language = Language::Python;
                    } else if current_section.to_lowercase().contains(" c ") {
                        ir.language = Language::C;
                    }
                } else if in_table {
                    if current_row.len() < table_headers.len() || table_headers.is_empty() {
                        if let Some(last) = current_row.last_mut() {
                            last.push_str(&t);
                        } else {
                            current_row.push(t);
                        }
                    } else {
                        if let Some(last) = current_row.last_mut() {
                            last.push_str(&t);
                        }
                    }
                } else if in_code_block {
                    current_code.push_str(&t);
                } else if in_list {
                    current_list_item.push_str(&t);
                } else if in_paragraph {
                    current_paragraph.push_str(&t);
                } else {
                    current_section.push_str(&t);
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                // v0.0.31 Session 12: Heading에서 컴포넌트 파일명(예: database.c/h, main.c 등) 정밀 추출!
                let sec_lower = current_section.to_lowercase();
                if sec_lower.contains(".c") || sec_lower.contains(".h") || sec_lower.contains(".rs") || sec_lower.contains(".py") || sec_lower.contains(".cpp") {
                    let mut file_candidate = String::new();
                    for part in current_section.split_whitespace() {
                        let clean_part = part.trim_matches(|c| c == '`' || c == '(' || c == ')' || c == ',' || c == ':');
                        if clean_part.contains(".c") || clean_part.contains(".h") || clean_part.contains(".rs") || clean_part.contains(".py") || clean_part.contains(".cpp") {
                            let clean_file = clean_file_path(clean_part);
                            file_candidate = clean_file;
                            break;
                        }
                    }
                    
                    if !file_candidate.is_empty() {
                        let node_id = file_candidate.split('.').next().unwrap_or("").to_uppercase().to_string();
                        if !node_id.is_empty() {
                            _active_component_file = file_candidate.clone();
                            active_component_node_id = node_id.clone();
                            
                            if !ir.nodes.iter().any(|n| n.id == node_id) {
                                ir.nodes.push(LogicalNode {
                                    id: node_id,
                                    tier: "component".to_string(),
                                    file_path: file_candidate,
                                    description: current_section.clone(),
                                });
                            }
                        }
                    }
                }
            }
            Event::Start(Tag::Table(_)) => {
                in_table = true;
                table_headers.clear();
            }
            Event::End(TagEnd::Table) => {
                in_table = false;
            }
            Event::Start(Tag::TableHead) => {
                current_row.clear();
            }
            Event::End(TagEnd::TableHead) => {
                table_headers = current_row.clone();
                current_row.clear();
            }
            Event::Start(Tag::TableRow) => {
                current_row.clear();
            }
            Event::Start(Tag::TableCell) => {
                current_row.push(String::new());
            }
            Event::End(TagEnd::TableRow) => {
                if !table_headers.is_empty() && current_row.len() >= 2 {
                    if current_section.to_lowercase().contains("mapping") || current_section.to_lowercase().contains("node") || current_section.to_lowercase().contains("component") {
                        let node_type = current_row.get(0).unwrap_or(&String::new()).trim().to_string();
                        let target_file = current_row.get(1).unwrap_or(&String::new()).trim().to_string();
                        let signature = current_row.get(2).unwrap_or(&String::new()).trim().to_string();

                        if !node_type.is_empty() {
                            let clean_node_id = node_type.trim_matches(|c| c == '`' || c == '*').to_string();
                            
                            if !ir.nodes.iter().any(|n| n.id == clean_node_id) {
                                ir.nodes.push(LogicalNode {
                                    id: clean_node_id.clone(),
                                    tier: "component".to_string(),
                                    file_path: target_file,
                                    description: String::new(),
                                });
                            }

                            if !signature.is_empty() {
                                let sym = signature.split('(').next().unwrap_or("").split_whitespace().last().unwrap_or("").to_string();
                                ir.interfaces.push(InterfaceContract {
                                    node_id: clean_node_id,
                                    symbol: sym,
                                    signature,
                                });
                            }
                        }
                    }
                }
            }
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
                current_code.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                if current_section.to_lowercase().contains("transition") || current_code.contains("graph TD") || current_code.contains("flowchart") {
                    parse_mermaid_transitions(&current_code, &mut ir.transitions, &mut ir.retry_loops);
                }
            }
            Event::Start(Tag::Item) => {
                in_list = true;
                current_list_item.clear();
            }
            Event::End(TagEnd::Item) => {
                in_list = false;
                
                // v0.0.31 Session 12: 만약 활성 컴포넌트가 있고, 아이템이 함수 선언문 양식이면 인터페이스 계약 자동 등록!
                if !active_component_node_id.is_empty() {
                    let item_trim = current_list_item.trim();
                    if item_trim.starts_with('`') || item_trim.contains('(') {
                        let sig = if item_trim.starts_with('`') {
                            if let Some(end_idx) = item_trim[1..].find('`') {
                                item_trim[1..end_idx + 1].trim().to_string()
                            } else {
                                item_trim.to_string()
                            }
                        } else {
                            item_trim.split(':').next().unwrap_or(item_trim).trim().to_string()
                        };
                        
                        if sig.contains('(') && !sig.is_empty() {
                            let sym = sig.split('(').next().unwrap_or("")
                                .split_whitespace().last().unwrap_or("")
                                .trim_matches(|c| c == '*' || c == '&' || c == '`')
                                .to_string();
                                
                            if !sym.is_empty() && sym != "fn" && sym != "pub" {
                                ir.interfaces.push(InterfaceContract {
                                    node_id: active_component_node_id.clone(),
                                    symbol: sym,
                                    signature: sig,
                                });
                            }
                        }
                    }
                }

                if current_section.to_lowercase().contains("invariant") || current_section.to_lowercase().contains("불변") {
                    let parts: Vec<&str> = current_list_item.split(':').collect();
                    if parts.len() >= 2 {
                        ir.invariants.push(ArchitecturalInvariant {
                            id: parts[0].trim().to_string(),
                            rule: parts[1..].join(":").trim().to_string(),
                            message: "Invariant violated".to_string(),
                        });
                    }
                } else if current_section.to_lowercase().contains("dependenc") || current_section.to_lowercase().contains("의존성") || current_section.to_lowercase().contains("constraint") {
                    ir.dependencies.push(DependencyConstraint {
                        target: current_list_item.split_whitespace().next().unwrap_or("").to_string(),
                        allowed_dependencies: vec![],
                        forbidden_dependencies: vec![],
                    });
                }
            }
            Event::Start(Tag::Paragraph) => {
                in_paragraph = true;
                current_paragraph.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                in_paragraph = false;
                if current_section.to_lowercase().contains("mapping") && current_paragraph.contains("NodeTypeTarget File") {
                    extract_fallback_nodes(&current_paragraph, &mut ir.nodes, &mut ir.interfaces);
                }
            }
            _ => {}
        }
    }

    ir
}

fn parse_mermaid_transitions(code: &str, transitions: &mut Vec<StateTransition>, retry_loops: &mut Vec<RetryLoop>) {
    for line in code.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("graph") || line.starts_with("flowchart") {
            continue;
        }

        let parts: Vec<&str> = line.split("-->").collect();
        if parts.len() == 2 {
            let from = parts[0].trim().to_string();
            let right = parts[1].trim();

            let to;
            let mut condition = None;
            let mut kind = TransitionKind::Forward;

            if right.starts_with('|') {
                if let Some(end_idx) = right[1..].find('|') {
                    let cond = &right[1..end_idx + 1];
                    condition = Some(cond.to_string());
                    to = right[end_idx + 2..].trim().to_string();

                    if cond.to_lowercase().contains("fail") || cond.to_lowercase().contains("err") {
                        kind = TransitionKind::Rollback;
                        retry_loops.push(RetryLoop {
                            trigger_node: from.clone(),
                            target_node: to.clone(),
                            condition: cond.to_string(),
                        });
                    } else {
                        kind = TransitionKind::Conditional;
                    }
                } else {
                    to = right.to_string();
                }
            } else {
                to = right.to_string();
            }

            transitions.push(StateTransition {
                from,
                to,
                condition,
                kind,
            });
        }
    }
}

fn extract_fallback_nodes(text: &str, nodes: &mut Vec<LogicalNode>, interfaces: &mut Vec<InterfaceContract>) {
    let regex = regex::Regex::new(r"([A-Z_]{3,})([a-z_]+)(src/[a-z_A-Z0-9./]+\.[A-Za-z]+)(pub fn [a-z_0-9]+\([^)]*\)(?: -> [a-zA-Z<>(), ]+)?|fn main\([^)]*\)[^A-Z]*|[a-zA-Z0-9_* ]+)").unwrap();
    for cap in regex.captures_iter(text) {
        let id = cap[1].to_string();
        let tier = cap[2].to_string();
        let path = cap[3].to_string();
        let sig = cap[4].to_string();

        nodes.push(LogicalNode {
            id: id.clone(),
            tier,
            file_path: path.clone(),
            description: String::new(),
        });

        let sym = sig.split('(').next().unwrap_or("").split_whitespace().last().unwrap_or("").to_string();
        interfaces.push(InterfaceContract {
            node_id: id,
            symbol: sym,
            signature: sig,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mermaid() {
        let mermaid = "graph TD\nSTART --> INPUT_NAME\nDB_CHECK -->|HIT| BYPASS\nVALID_YEAR -->|FAIL| INPUT_YEAR";
        let mut transitions = vec![];
        let mut retry = vec![];
        parse_mermaid_transitions(mermaid, &mut transitions, &mut retry);

        assert_eq!(transitions.len(), 3);
        assert_eq!(transitions[0].from, "START");
        assert_eq!(transitions[0].to, "INPUT_NAME");
        
        assert_eq!(transitions[2].kind, TransitionKind::Rollback);
        assert_eq!(retry.len(), 1);
        assert_eq!(retry[0].trigger_node, "VALID_YEAR");
    }
}
