use axon_platform_win32::{get_win32_contract, Win32Contract};
use regex::Regex;
use anyhow::{Result, bail};

pub struct PlatformValidator {
    contract: Win32Contract,
}

impl Default for PlatformValidator {
    fn default() -> Self {
        Self {
            contract: get_win32_contract(),
        }
    }
}

impl PlatformValidator {
    pub fn new() -> Self {
        Self::default()
    }

    /// 소스 파일의 내용(텍스트)을 분석하여 Win32 GUI 런타임 규약을 위반하는지 검증합니다.
    pub fn validate_source_code(&self, filename: &str, content: &str, is_entrypoint: bool) -> Result<()> {
        // C/C++ 소스 파일 또는 관련 코드에 한하여 검사 적용
        if !filename.ends_with(".c") && !filename.ends_with(".cpp") && !filename.ends_with(".h") {
            return Ok(());
        }

        // 0-A. Win32 Header Ownership & Fake Runtime Prevention
        let win32_types = [
            "HWND", "HDC", "HINSTANCE", "HMENU", "HBRUSH", "HCURSOR", "MSG", "RECT", 
            "PAINTSTRUCT", "WNDCLASS", "WNDCLASSEX", "WNDPROC", "LPARAM", "WPARAM", "LRESULT"
        ];
        
        let mut uses_win32 = false;
        for t_name in &win32_types {
            let re = Regex::new(&format!(r"\b{}\b", t_name))?;
            if re.is_match(content) {
                uses_win32 = true;
                break;
            }
        }

        let win32_calls = ["CreateWindow", "CreateWindowEx", "DefWindowProc", "BeginPaint", "EndPaint"];
        for c_name in &win32_calls {
            let re = Regex::new(&format!(r"\b{}\b", c_name))?;
            if re.is_match(content) {
                uses_win32 = true;
                break;
            }
        }

        if uses_win32 {
            if !content.contains("<windows.h>") && !content.contains("\"windows.h\"") {
                bail!("PLATFORM_CONTRACT_VIOLATION: Win32 type/API usage detected in {} but '#include <windows.h>' is missing. Win32 types are owned by windows.h and must not be used without including it.", filename);
            }

            let fake_typedefs = [
                r"\btypedef\s+.*\bHWND\b",
                r"\bstruct\s+HWND\b",
                r"\bstruct\s+HWND__\b",
                r"\bint\s+CreateWindow\b",
                r"\bint\s+CreateWindowEx\b",
                r"\bint\s+DefWindowProc\b",
                r"\btypedef\s+.*\bLRESULT\b",
                r"\btypedef\s+.*\bHINSTANCE\b",
                r"\btypedef\s+.*\bWPARAM\b",
                r"\btypedef\s+.*\bLPARAM\b",
                r"\bHWND\s+(WINAPI\s+)?CreateWindowEx\b",
                r"\bLRESULT\s+(CALLBACK\s+)?DispatchMessage\b",
                r"\bBOOL\s+(WINAPI\s+)?TranslateMessage\b",
                r"\bLRESULT\s+(CALLBACK\s+)?DefWindowProc\b",
                r"\bHDC\s+(WINAPI\s+)?BeginPaint\b",
                r"\bBOOL\s+(WINAPI\s+)?EndPaint\b",
                r"\bBOOL\s+(WINAPI\s+)?ShowWindow\b",
                r"\bBOOL\s+(WINAPI\s+)?UpdateWindow\b",
                r"\bBOOL\s+(WINAPI\s+)?GetMessage\b",
            ];
            for p in &fake_typedefs {
                let re = Regex::new(p)?;
                if re.is_match(content) {
                    bail!("PLATFORM_CONTRACT_VIOLATION: Fake Win32 type definition or API replacement detected in {}. LLM must treat Win32 types/APIs as external OS ABI entities and MUST NOT define or replace them.", filename);
                }
            }
        }

        // 0. OS System Library 파일 생성 원천 차단 (System Library Contract Layer)
        let filename_lower = filename.to_ascii_lowercase();
        let base_name = std::path::Path::new(&filename_lower)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        let system_libraries = ["user32", "gdi32", "kernel32", "shell32", "comdlg32", "gdi", "win32_api"];
        if system_libraries.contains(&base_name) {
            bail!("PLATFORM_CONTRACT_VIOLATION: System library '{}' cannot be generated as a project-owned module. It is a link-time OS service, not a source implementation.", filename);
        }

        // 1. Forbidden entry point check
        let main_re = Regex::new(r"\b(int|void)?\s*main\s*\(")?;
        if main_re.is_match(content) {
            bail!("PLATFORM_CONTRACT_VIOLATION: Forbidden entry point 'main' detected in {}. Win32 GUI application must use '{}' as the main entry point, and console 'main' is strictly forbidden.", 
                filename, self.contract.required_entry);
        }

        // 만약 이 파일이 아키텍처 상의 entrypoint 컴포넌트인데 wWinMain 이나 WinMain이 존재하지 않는 경우
        if is_entrypoint {
            let entry_re = Regex::new(r"\b(wWinMain|WinMain)\s*\(")?;
            if !entry_re.is_match(content) {
                bail!("PLATFORM_CONTRACT_VIOLATION: Entrypoint component '{}' lacks the mandatory Win32 GUI entrypoint function '{}' or 'WinMain'. Console entrypoints are forbidden.",
                    filename, self.contract.required_entry);
            }
        }

        // 2. Forbidden patterns check
        for pattern in &self.contract.forbidden_patterns {
            if content.contains(pattern) {
                bail!("PLATFORM_CONTRACT_VIOLATION: Forbidden library/pattern '{}' detected in {}. This is invalid for Win32 GUI App.", 
                    pattern, filename);
            }
        }
        for forbidden_call in &self.contract.forbidden_calls {
            let re = Regex::new(&format!(r"\b{}\b", forbidden_call))?;
            if re.is_match(content) {
                bail!("PLATFORM_CONTRACT_VIOLATION: Forbidden API call '{}' detected in {}.", 
                    forbidden_call, filename);
            }
        }

        // 3. Required components check (wWinMain 이나 WinMain이 정의된 메인 진입점 파일)
        let entry_check_re = Regex::new(r"\b(wWinMain|WinMain)\s*\(")?;
        if entry_check_re.is_match(content) {
            // GetMessage 루프 검사
            for req_call in &self.contract.required_calls {
                let call_re = Regex::new(&format!(r"\b{}\s*\(", req_call))?;
                if !call_re.is_match(content) {
                    bail!("PLATFORM_CONTRACT_VIOLATION: Main source code lacks required Win32 API call '{}' in message loop. A fake render loop or polling is forbidden.", req_call);
                }
            }

            // WndProc 필수 메시지 처리 검사
            for req_msg in &self.contract.required_messages {
                let msg_re = Regex::new(&format!(r"\b{}\b", req_msg))?;
                if !msg_re.is_match(content) {
                    bail!("PLATFORM_CONTRACT_VIOLATION: Win32 WndProc lacks mandatory message handler: {}.", req_msg);
                }
            }

            let def_wnd_re = Regex::new(r"\bDefWindowProc\s*\(")?;
            if !def_wnd_re.is_match(content) {
                bail!("PLATFORM_CONTRACT_VIOLATION: Win32 WndProc lacks mandatory callback chain delegation to 'DefWindowProc'.");
            }
        }

        // 4. Message Runtime Validation: Rendering only inside WM_PAINT
        if content.contains("BeginPaint") || content.contains("EndPaint") {
            let wm_paint_re = Regex::new(r"\bWM_PAINT\b")?;
            if !wm_paint_re.is_match(content) {
                bail!("PLATFORM_CONTRACT_VIOLATION: Rendering call (BeginPaint/EndPaint) detected but WM_PAINT message handler is missing.");
            }

            if let Some(wm_paint_pos) = content.find("WM_PAINT") {
                let post_wm_paint = &content[wm_paint_pos..];
                let end_boundary = post_wm_paint.find("case ")
                    .or_else(|| post_wm_paint.find("default:"))
                    .unwrap_or(post_wm_paint.len());
                let wm_paint_block = &post_wm_paint[..end_boundary];

                let total_begin = content.matches("BeginPaint").count();
                let block_begin = wm_paint_block.matches("BeginPaint").count();
                if total_begin != block_begin {
                    bail!("PLATFORM_CONTRACT_VIOLATION: Continuous rendering or rendering outside WM_PAINT is forbidden. 'BeginPaint' must only be called inside the WM_PAINT message handler scope.");
                }

                let total_end = content.matches("EndPaint").count();
                let block_end = wm_paint_block.matches("EndPaint").count();
                if total_end != block_end {
                    bail!("PLATFORM_CONTRACT_VIOLATION: Continuous rendering or rendering outside WM_PAINT is forbidden. 'EndPaint' must only be called inside the WM_PAINT message handler scope.");
                }
            } else {
                bail!("PLATFORM_CONTRACT_VIOLATION: BeginPaint/EndPaint is called but WM_PAINT scope cannot be verified.");
            }
        }

        Ok(())
    }

    /// 빌드된 바이너리/Executable의 Subsystem 속성이 WINDOWS_GUI(2)인지 검증 (PE Subsystem 검사)
    pub fn validate_binary_subsystem(&self, binary_path: &str) -> Result<()> {
        let bytes = std::fs::read(binary_path)?;
        if bytes.len() < 64 {
            bail!("Invalid binary format: too short");
        }
        
        // PE signature offset
        let pe_offset = u32::from_le_bytes([bytes[0x3c], bytes[0x3d], bytes[0x3e], bytes[0x3f]]) as usize;
        if bytes.len() < pe_offset + 4 {
            bail!("Invalid PE binary offset");
        }
        
        // Check "PE\0\0"
        if bytes[pe_offset] != b'P' || bytes[pe_offset+1] != b'E' || bytes[pe_offset+2] != 0 || bytes[pe_offset+3] != 0 {
            bail!("PLATFORM_CONTRACT_VIOLATION: Binary is not a Portable Executable (PE). Ensure compilation targets MinGW/Windows.");
        }
        
        // Optional header starts at pe_offset + 24
        let opt_header_offset = pe_offset + 24;
        if bytes.len() < opt_header_offset + 2 {
            bail!("Invalid PE optional header");
        }
        
        let magic = u16::from_le_bytes([bytes[opt_header_offset], bytes[opt_header_offset+1]]);
        let subsystem_offset = match magic {
            0x10b => opt_header_offset + 68, // PE32
            0x20b => opt_header_offset + 68, // PE32+
            _ => bail!("Unknown PE optional header magic"),
        };
        
        if bytes.len() < subsystem_offset + 2 {
            bail!("Invalid PE subsystem offset");
        }
        
        let subsystem = u16::from_le_bytes([bytes[subsystem_offset], bytes[subsystem_offset+1]]);
        if subsystem != 2 {
            bail!("PLATFORM_CONTRACT_VIOLATION: Subsystem is not WINDOWS_GUI (expected 2, got {}). Binary will open a Console Window.", subsystem);
        }

        // user32 및 gdi32 임포트 링키지 강제 검사
        let bytes_lower = bytes.to_ascii_lowercase();
        
        let has_user32 = bytes_lower.windows(10).any(|w| w == b"user32.dll");
        if !has_user32 {
            bail!("PLATFORM_CONTRACT_VIOLATION: Binary lacks linkage to 'user32.dll'. Ensure GUI windowing library is properly linked.");
        }

        let has_gdi32 = bytes_lower.windows(9).any(|w| w == b"gdi32.dll");
        if !has_gdi32 {
            bail!("PLATFORM_CONTRACT_VIOLATION: Binary lacks linkage to 'gdi32.dll'. Ensure GDI drawing library is properly linked.");
        }

        Ok(())
    }

    /// Skeleton 설계 단계 전에 명세(spec.md)의 원본 및 제약 조건을 분석하여 플랫폼 모순 여부를 검증하는 Runtime Validation Pass.
    pub fn validate_spec(&self, spec_content: &str, constraints: &axon_core::spec::ImmutableConstraints) -> Result<()> {
        let is_win32 = constraints.platform.as_deref() == Some("win32")
            || constraints.runtime_model.as_deref() == Some("win32_gui")
            || spec_content.to_lowercase().contains("win32");

        if is_win32 {
            // 1. Language validation: Win32 GUI requires C/C++
            let lang = constraints.language.to_lowercase();
            if lang != "c" && lang != "cpp" && lang != "c++" {
                bail!("PLATFORM_CONTRACT_VIOLATION: Win32 GUI applications require C or C++ programming language (expected 'c' or 'cpp', got '{}').",
                    constraints.language);
            }

            // 2. Spec content forbidden patterns scanning
            for pattern in &self.contract.forbidden_patterns {
                let pattern_lower = pattern.to_lowercase();
                if spec_content.to_lowercase().contains(&pattern_lower) {
                    bail!("PLATFORM_CONTRACT_VIOLATION: Specification requests forbidden cross-platform GUI/Console library/pattern '{}' for Win32 GUI app.", 
                        pattern);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_source_code_main_forbidden() {
        let val = PlatformValidator::new();
        // int main 이 정의된 C 파일은 forbidden main 에러를 반환해야 합니다.
        let code1 = "int main(int argc, char** argv) { return 0; }";
        let res = val.validate_source_code("src/main.c", code1, false);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("PLATFORM_CONTRACT_VIOLATION: Forbidden entry point 'main'"));

        let code2 = "void main() {}";
        let res2 = val.validate_source_code("src/main.c", code2, false);
        assert!(res2.is_err());
    }

    #[test]
    fn test_validate_source_code_entrypoint_missing() {
        let val = PlatformValidator::new();
        // entrypoint 컴포넌트인데 wWinMain / WinMain 이 없으면 에러를 반환해야 합니다.
        let code = "void some_helper_function() {}";
        let res = val.validate_source_code("src/main.c", code, true);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("lacks the mandatory Win32 GUI entrypoint"));
    }

    #[test]
    fn test_validate_source_code_message_loop_missing() {
        let val = PlatformValidator::new();
        // wWinMain 은 있으나 GetMessage, DispatchMessage 메시지 펌프가 누락된 경우
        let code = r#"
            #include <windows.h>
            int WINAPI wWinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPWSTR lpCmdLine, int nCmdShow) {
                // Fake loop
                while(1) {}
            }
        "#;
        let res = val.validate_source_code("src/main.c", code, true);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("lacks required Win32 API call"));
    }

    #[test]
    fn test_validate_source_code_wndproc_defwndproc_missing() {
        let val = PlatformValidator::new();
        // wWinMain 이 있고 메시지 루프도 있지만 DefWindowProc 콜백 위임이 누락된 경우
        let code = r#"
            #include <windows.h>
            int WINAPI wWinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPWSTR lpCmdLine, int nCmdShow) {
                MSG msg;
                while(GetMessage(&msg, NULL, 0, 0)) {
                    TranslateMessage(&msg);
                    DispatchMessage(&msg);
                }
                return 0;
            }
            LRESULT CALLBACK WndProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam) {
                switch(message) {
                    case WM_PAINT:
                        break;
                }
                return 0; // DefWndProc missing!
            }
        "#;
        let res = val.validate_source_code("src/main.c", code, true);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("lacks mandatory callback chain delegation to 'DefWindowProc'"));
    }

    #[test]
    fn test_validate_source_code_valid() {
        let val = PlatformValidator::new();
        // 정상적인 wWinMain, GetMessage, WndProc(WM_PAINT + DefWindowProc) 구조는 통과해야 합니다.
        let code = r#"
            #include <windows.h>
            int WINAPI wWinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPWSTR lpCmdLine, int nCmdShow) {
                MSG msg;
                while(GetMessage(&msg, NULL, 0, 0)) {
                    TranslateMessage(&msg);
                    DispatchMessage(&msg);
                }
                return 0;
            }
            LRESULT CALLBACK WndProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam) {
                switch(message) {
                    case WM_PAINT: {
                        PAINTSTRUCT ps;
                        HDC hdc = BeginPaint(hWnd, &ps);
                        EndPaint(hWnd, &ps);
                        break;
                    }
                    default:
                        return DefWindowProc(hWnd, message, wParam, lParam);
                }
                return 0;
            }
        "#;
        let res = val.validate_source_code("src/main.c", code, true);
        assert!(res.is_ok(), "Error: {:?}", res);
    }

    #[test]
    fn test_validate_source_code_rendering_outside_wm_paint() {
        let val = PlatformValidator::new();
        // BeginPaint를 WM_PAINT 외부(예: WndProc의 WM_CREATE 블록)에서 오용한 코드
        let code = r#"
            #include <windows.h>
            int WINAPI wWinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPWSTR lpCmdLine, int nCmdShow) {
                MSG msg;
                while(GetMessage(&msg, NULL, 0, 0)) {
                    TranslateMessage(&msg);
                    DispatchMessage(&msg);
                }
                return 0;
            }
            LRESULT CALLBACK WndProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam) {
                switch(message) {
                    case WM_CREATE: {
                        PAINTSTRUCT ps;
                        HDC hdc = BeginPaint(hWnd, &ps); // Wrong place!
                        EndPaint(hWnd, &ps);
                        break;
                    }
                    case WM_PAINT:
                        break;
                    default:
                        return DefWindowProc(hWnd, message, wParam, lParam);
                }
                return 0;
            }
        "#;
        let res = val.validate_source_code("src/main.c", code, true);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Continuous rendering or rendering outside WM_PAINT is forbidden"));
    }

    #[test]
    fn test_validate_binary_subsystem_invalid() {
        let val = PlatformValidator::new();
        // 1. 너무 짧은 바이너리 에러 검증
        let invalid_short = vec![0u8; 10];
        let temp_dir = std::env::temp_dir();
        let path1 = temp_dir.join("test_short.exe");
        std::fs::write(&path1, invalid_short).unwrap();
        let res1 = val.validate_binary_subsystem(path1.to_str().unwrap());
        assert!(res1.is_err());
        assert!(res1.unwrap_err().to_string().contains("too short"));

        // 2. PE 헤더 부재 에러 검증
        let invalid_no_pe = vec![0u8; 100];
        let path2 = temp_dir.join("test_no_pe.exe");
        std::fs::write(&path2, invalid_no_pe).unwrap();
        let res2 = val.validate_binary_subsystem(path2.to_str().unwrap());
        assert!(res2.is_err());
        assert!(res2.unwrap_err().to_string().contains("Ensure compilation targets MinGW/Windows"));
        
        let _ = std::fs::remove_file(path1);
        let _ = std::fs::remove_file(path2);
    }

    #[test]
    fn test_validate_binary_subsystem_valid_and_linkage_checks() {
        let val = PlatformValidator::new();
        
        // 정상적인 PE 구조를 모방하되 user32.dll과 gdi32.dll 링키지가 누락된 케이스와 통과 케이스 구축
        let mut binary = vec![0u8; 500];
        // MZ 헤더 시그니처 및 PE 오프셋 설정
        binary[0] = b'M';
        binary[1] = b'Z';
        binary[0x3c] = 100; // pe_offset
        binary[0x3d] = 0;
        binary[0x3e] = 0;
        binary[0x3f] = 0;

        // PE 시그니처 주입
        binary[100] = b'P';
        binary[101] = b'E';
        binary[102] = 0;
        binary[103] = 0;

        // Optional Magic (PE32) -> 0x10b
        // opt_header_offset = pe_offset + 24 = 124
        binary[124] = 0x0b;
        binary[125] = 0x01;

        // Subsystem offset for PE32 = opt_header_offset + 68 = 192
        // Subsystem value (WINDOWS_GUI = 2)
        binary[192] = 2;
        binary[193] = 0;

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_mock.exe");

        // A. 링키지 누락 케이스
        std::fs::write(&path, &binary).unwrap();
        let res = val.validate_binary_subsystem(path.to_str().unwrap());
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("lacks linkage to 'user32.dll'"));

        // B. user32.dll 만 존재하고 gdi32.dll 누락된 케이스
        let mut binary_half = binary.clone();
        binary_half.extend_from_slice(b"user32.dll");
        std::fs::write(&path, &binary_half).unwrap();
        let res_half = val.validate_binary_subsystem(path.to_str().unwrap());
        assert!(res_half.is_err());
        assert!(res_half.unwrap_err().to_string().contains("lacks linkage to 'gdi32.dll'"));

        // C. 둘 다 정상 링키지 케이스
        let mut binary_full = binary.clone();
        binary_full.extend_from_slice(b"user32.dll");
        binary_full.extend_from_slice(b"gdi32.dll");
        std::fs::write(&path, &binary_full).unwrap();
        let res_full = val.validate_binary_subsystem(path.to_str().unwrap());
        assert!(res_full.is_ok(), "Error: {:?}", res_full);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_validate_source_code_system_library_forbidden() {
        let val = PlatformValidator::new();
        // user32.c 파일 생성 시도 -> 기각 확인
        let res = val.validate_source_code("src/user32.c", "void foo() {}", false);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("cannot be generated as a project-owned module"));

        // gdi.c 파일 생성 시도 -> 기각 확인
        let res2 = val.validate_source_code("gdi.c", "void foo() {}", false);
        assert!(res2.is_err());
        assert!(res2.unwrap_err().to_string().contains("cannot be generated as a project-owned module"));

        // win32_api.h 파일 생성 시도 -> 기각 확인
        let res_api = val.validate_source_code("include/win32_api.h", "void foo() {}", false);
        assert!(res_api.is_err());
        assert!(res_api.unwrap_err().to_string().contains("cannot be generated as a project-owned module"));

        // normal.c 파일 생성 시도 -> 통과 확인
        let res3 = val.validate_source_code("src/normal.c", "void foo() {}", false);
        assert!(res3.is_ok());
    }

    #[test]
    fn test_validate_source_code_win32_header_ownership() {
        let val = PlatformValidator::new();
        
        // HWND가 있으나 windows.h 가 없는 케이스 -> 기각 확인
        let code_no_h = "HWND main_window;";
        let res1 = val.validate_source_code("src/main.c", code_no_h, false);
        assert!(res1.is_err());
        assert!(res1.unwrap_err().to_string().contains("Win32 type/API usage detected in src/main.c but '#include <windows.h>' is missing"));

        // HWND가 있고 windows.h 도 있는 케이스 -> 통과 확인
        let code_with_h = "#include <windows.h>\nHWND main_window;";
        let res2 = val.validate_source_code("src/main.c", code_with_h, false);
        assert!(res2.is_ok());
    }

    #[test]
    fn test_validate_source_code_fake_win32_detection() {
        let val = PlatformValidator::new();
        
        // HWND fake 정의 시도 -> 기각 확인
        let code_fake_typedef = "#include <windows.h>\ntypedef void* HWND;";
        let res1 = val.validate_source_code("src/main.c", code_fake_typedef, false);
        assert!(res1.is_err());
        assert!(res1.unwrap_err().to_string().contains("Fake Win32 type definition or API replacement detected"));

        // CreateWindow fake 정의 시도 -> 기각 확인
        let code_fake_fn = "#include <windows.h>\nint CreateWindow(int x) { return 0; }";
        let res2 = val.validate_source_code("src/main.c", code_fake_fn, false);
        assert!(res2.is_err());
        assert!(res2.unwrap_err().to_string().contains("Fake Win32 type definition or API replacement detected"));

        // CreateWindowEx 직접 선언 시도 -> 기각 확인
        let code_fake_createwindowex = "#include <windows.h>\nHWND WINAPI CreateWindowEx(DWORD dwExStyle, LPCSTR lpClassName, LPCSTR lpWindowName, DWORD dwStyle, int X, int Y, int nWidth, int nHeight, HWND hWndParent, HMENU hMenu, HINSTANCE hInstance, LPVOID lpParam);";
        let res3 = val.validate_source_code("src/main.c", code_fake_createwindowex, false);
        assert!(res3.is_err());
        assert!(res3.unwrap_err().to_string().contains("Fake Win32 type definition or API replacement detected"));

        // DispatchMessage 직접 선언 시도 -> 기각 확인
        let code_fake_dispatch = "#include <windows.h>\nLRESULT CALLBACK DispatchMessage(const MSG *lpMsg);";
        let res4 = val.validate_source_code("src/main.c", code_fake_dispatch, false);
        assert!(res4.is_err());
        assert!(res4.unwrap_err().to_string().contains("Fake Win32 type definition or API replacement detected"));
    }
}

