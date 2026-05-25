use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Win32Contract {
    pub required_entry: String,
    pub forbidden_entry: String,
    pub required_calls: Vec<String>,
    pub forbidden_calls: Vec<String>,
    pub required_messages: Vec<String>,
    pub forbidden_patterns: Vec<String>,
    pub expected_subsystem: String,
    
    // Embedded contract file contents for validation
    pub subsystem_contract: String,
    pub winmain_contract: String,
    pub message_loop_contract: String,
    pub wndproc_contract: String,
    pub rendering_contract: String,
}

const SUBSYSTEM_CONTRACT_RAW: &str = include_str!("../runtime_contracts/subsystem.contract");
const WINMAIN_CONTRACT_RAW: &str = include_str!("../runtime_contracts/winmain.contract");
const MESSAGE_LOOP_CONTRACT_RAW: &str = include_str!("../runtime_contracts/message_loop.contract");
const WNDPROC_CONTRACT_RAW: &str = include_str!("../runtime_contracts/wndproc.contract");
const RENDERING_CONTRACT_RAW: &str = include_str!("../runtime_contracts/rendering.contract");

fn parse_line_value(raw: &str, key: &str) -> Option<String> {
    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(pos) = line.find(':') {
            let k = line[..pos].trim();
            if k.eq_ignore_ascii_case(key) {
                return Some(line[pos+1..].trim().to_string());
            }
        }
    }
    None
}

fn parse_csv_value(raw: &str, key: &str) -> Vec<String> {
    if let Some(val) = parse_line_value(raw, key) {
        val.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    }
}

impl Default for Win32Contract {
    fn default() -> Self {
        let mut contract = Self {
            required_entry: "wWinMain".to_string(),
            forbidden_entry: "main".to_string(),
            required_calls: Vec::new(),
            forbidden_calls: Vec::new(),
            required_messages: Vec::new(),
            forbidden_patterns: Vec::new(),
            expected_subsystem: "WINDOWS".to_string(),
            subsystem_contract: SUBSYSTEM_CONTRACT_RAW.to_string(),
            winmain_contract: WINMAIN_CONTRACT_RAW.to_string(),
            message_loop_contract: MESSAGE_LOOP_CONTRACT_RAW.to_string(),
            wndproc_contract: WNDPROC_CONTRACT_RAW.to_string(),
            rendering_contract: RENDERING_CONTRACT_RAW.to_string(),
        };

        // 1. Subsystem Contract 파싱
        if let Some(sub) = parse_line_value(SUBSYSTEM_CONTRACT_RAW, "EXPECTED_SUBSYSTEM") {
            contract.expected_subsystem = sub;
        }
        contract.forbidden_patterns.extend(parse_csv_value(SUBSYSTEM_CONTRACT_RAW, "FORBIDDEN_PATTERNS"));

        // 2. WinMain Contract 파싱
        if let Some(entry) = parse_line_value(WINMAIN_CONTRACT_RAW, "REQUIRED_ENTRY") {
            contract.required_entry = entry;
        }
        if let Some(forbid) = parse_line_value(WINMAIN_CONTRACT_RAW, "FORBIDDEN_ENTRY") {
            contract.forbidden_entry = forbid;
        }

        // 3. Message Loop Contract 파싱
        contract.required_calls.extend(parse_csv_value(MESSAGE_LOOP_CONTRACT_RAW, "REQUIRED_CALLS"));
        contract.forbidden_calls.extend(parse_csv_value(MESSAGE_LOOP_CONTRACT_RAW, "FORBIDDEN_CALLS"));

        // 5. Rendering Contract 파싱
        contract.required_messages.extend(parse_csv_value(RENDERING_CONTRACT_RAW, "REQUIRED_MESSAGES"));

        // default fallback 및 확장 패턴 보강
        if contract.required_calls.is_empty() {
            contract.required_calls = vec![
                "GetMessage".to_string(),
                "TranslateMessage".to_string(),
                "DispatchMessage".to_string(),
            ];
        }
        if contract.forbidden_calls.is_empty() {
            contract.forbidden_calls = vec![
                "polling_loop".to_string(),
                "busy_renderer".to_string(),
                "frame_scheduler".to_string(),
            ];
        }
        if contract.required_messages.is_empty() {
            contract.required_messages = vec!["WM_PAINT".to_string()];
        }
        if contract.forbidden_patterns.is_empty() {
            contract.forbidden_patterns = vec![
                "SDL_Init".to_string(),
                "SDL_PollEvent".to_string(),
                "SDL_WaitEvent".to_string(),
                "glfwInit".to_string(),
                "glfwPollEvents".to_string(),
                "glfwWaitEvents".to_string(),
                "gtk_init".to_string(),
                "gtk_main".to_string(),
                "gtk_main_quit".to_string(),
                "QApplication".to_string(),
                "ncurses".to_string(),
                "initscr".to_string(),
                "electron".to_string(),
                "NW.js".to_string(),
            ];
        } else {
            // cross-platform library detection만 추가 (loop shape 검출은 제거)
            contract.forbidden_patterns.push("SDL_PollEvent".to_string());
            contract.forbidden_patterns.push("SDL_WaitEvent".to_string());
            contract.forbidden_patterns.push("glfwPollEvents".to_string());
            contract.forbidden_patterns.push("glfwWaitEvents".to_string());
            contract.forbidden_patterns.push("gtk_main".to_string());
        }

        contract
    }
}

pub fn get_win32_contract() -> Win32Contract {
    Win32Contract::default()
}
