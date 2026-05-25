# encoding: utf-8
MyLightIDE v0.0.1 — LINUX NATIVE GTK4 COMPLIANT SPEC

Identity: Native Linux GTK4 GUI System Framework Transition Specification.

## 0. PURPOSE
This document exists ONLY to help AXON, IR generators, validators, low-end LLMs, and code synthesis systems correctly generate a native Linux GTK4 GUI interface mapping for the MyLightIDE application.

## 1. PRIMARY RULE
IMPORTANT. The generated application MUST be:
```text
A native Linux GTK4 application.
```
GTK version: GTK4 ONLY (gtk4 / libgtk-4-dev)
STRICTLY FORBIDDEN:
* Windows GUI application
* SDL application
* Electron application
* Cross-platform abstraction wrappers.
* Terminal-only rendering engines.

## 2. REQUIRED RUNTIME TARGET
Target OS: Linux
Target ABI: POSIX / Linux ABI
Target Output: ELF (Executable and Linkable Format)
Target Type: GTK4 Window application

## 3. REQUIRED ENTRY POINT
The application MUST use the standard POSIX compliant entry signature:
```cpp
int main(int argc, char** argv)
```

## 4. FORBIDDEN ENTRY POINTS
The following are FORBIDDEN:
```cpp
WinMain()
wWinMain()
```
Reason: These imply Windows PE subsystem generation.

## 7. IMPORTANT API & GTK4 OWNERSHIP CONTRACT
IMPORTANT. The following frameworks are OS-provided system dependencies. They are NOT user-generated implementation code.
```text
libc
pthread
gtk4          ← GTK4 ONLY. gtk+-3.0 (GTK3) 사용 금지.
glib-2.0
cairo         ← GTK4의 핵심 드로잉 API. 허용 대상.
```
Therefore, generating local source modules such as src/libc.cpp, src/gtk.cpp, or src/cairo.cpp is strictly banned.

## 21. FORBIDDEN RUNTIME CONTEXT
Forbidden:
```text
Win32 message queue loop
GetMessage()
DispatchMessage()
WndProc callback pattern
```
Reason: This is not a Linux GTK4 architecture.

Required Event Loop (GTK4):
```cpp
// GTK4 표준 진입점
GtkApplication* app = gtk_application_new("org.mylightide", G_APPLICATION_DEFAULT_FLAGS);
g_signal_connect(app, "activate", G_CALLBACK(on_activate), NULL);
return g_application_run(G_APPLICATION(app), argc, argv);
```
NOTE: `gtk_main()` is deprecated in GTK4. Use `g_application_run()` only.

## 24. FORBIDDEN RENDERING SCHEMES
Forbidden:
```text
OpenGL
Vulkan
DirectX
Chromium renderer
Win32 GDI
ncurses
termios console modes
```
Allowed (GTK4 렌더링 스택):
```text
GtkDrawingArea + Cairo  ← 커스텀 렌더링 (추천)
GtkTextView             ← 단순 텍스트 영역 (폴백 목적)
Pango                   ← 텍스트 레이아웃 / 폰트
```

## 40. REQUIRED BUILD TOOLCHAIN & CMAKE STRUCTURE

### 컴파일러 확정
```text
Compiler : GCC 12+  (Clang 사용 금지)
Standard : C++17
Target   : Linux ELF (x86_64)
```

### CMakeLists.txt 고정 명세 (GTK4 >= 4.6)
```cmake
cmake_minimum_required(VERSION 3.16)
project(MyLightIDE)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_COMPILER g++)   # GCC 고정

# GTK4 전용 — gtk+-3.0 (GTK3) 사용 금지
# 최소 버전 4.6 이상 강제
find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK4 REQUIRED IMPORTED_TARGET "gtk4 >= 4.6")

include_directories(include)

add_executable(MyLightIDE
    src/main.cpp
    src/gtk_app.cpp
    src/text_buffer.cpp
    src/lua_runtime.cpp
    src/input.cpp
    src/terminal_renderer.cpp
)

# System dependencies are link-targets only! No local implementation files allowed.
target_link_libraries(MyLightIDE
    PkgConfig::GTK4
    pthread
    dl
    m
    stdc++fs
)
```

### 빌드 시퀀스
```bash
# 1단계: 빌드 디렉토리 생성
mkdir -p build && cd build

# 2단계: 설정 (Debug 모드)
cmake .. -DCMAKE_BUILD_TYPE=Debug

# 3단계: 컴파일
make -j$(nproc)

# 4단계: 실행
./MyLightIDE
```

## 42. REQUIRED VALIDATOR CONTROLS
Validator는 다음 두 카테고리를 즉시 거부해야 한다.

[A] Win32 심볼 침투 → 즉시 거부:
```text
WinMain()
wWinMain()
RegisterClassExW
CreateWindowExW
HWND / HINSTANCE / WPARAM / LPARAM
```

[B] GTK3 전용 deprecated API → 경고 후 거부 (GTK4와 비호환):
```text
gtk_widget_show_all()   ← GTK4에서 제거됨
gtk_init(&argc, &argv)  ← GTK4는 인자 없음: gtk_init()
gtk_main()              ← GTK4에서 deprecated
gtk+-3.0 헤더 include    ← GTK3 전용 헤더
```

---

## 48. GTK4 WIDGET ARCHITECTURE MAP
GEMINI.md에 정의된 UI 구조와 GTK4 위젯 매핑.
AXON과 LLM은 이 매핑을 반드시 준수해야 한다.

```text
UI 구성요소             GTK4 위젯
─────────────────────────────────────────────
메인 윈도우          →  GtkApplicationWindow
수평 컨테이너        →  GtkBox (horizontal, spacing=0)
액티비티 바 (2버튼)  →  GtkBox (vertical) + GtkToggleButton x2
사이드바 토글        →  GtkRevealer (transition: slide-right)
스플리터 바         →  GtkPaned (orientation: horizontal)
메인 에디터        →  GtkDrawingArea (draw_func 콜백)
상태바               →  GtkLabel (하단 고정)
파일 트리탐색기     →  GtkListView + GtkTreeListModel
확장 매니저       →  GtkListBox (하드코딩 6개 항목)
```

### 레이아웃 계층 구조
```
GtkApplicationWindow
  └─ GtkBox (horizontal)
      ├─ GtkBox (vertical, activity-bar)   ← 액티비티 바
      ├─ GtkRevealer                        ← 사이드바 (토글)
      │   └─ GtkStack
      │       ├─ GtkListView (파일 트리)
      │       └─ GtkListBox (확장 리스트)
      ├─ GtkPaned                           ← 스플리터 (자동 내장)
      │   └─ GtkDrawingArea                 ← 메인 에디터
      └─ GtkLabel                           ← 상태바
```

---

## 49. GTK4 EVENT CONTRACT (Lua 브릿지)
GTK4는 GTK3의 시그널 방식(`key-press-event`)을 포기하고 `GtkEventController` 체계로 교체했다.
AXON은 GTK3 시그널 연결 방식을 **절대 사용하지 마라.**

### 이벤트 주체 원칙
```text
GTK4 이벤트              →  C++ 함수      →  Lua 콜백
─────────────────────────────────────────────────────────
GtkEventControllerKey  →  on_key_press()  →  lua_call("on_key")
GtkDrawingArea draw    →  draw_func()     →  lua_call("on_render")
GtkGestureClick        →  on_click()      →  lua_call("on_click")
```

### GTK4 키입력 연결 (C++ 코드 패턴)
```cpp
// 직접 시그널 연결 금지 — EventController 만 사용
GtkEventController* key_ctrl = gtk_event_controller_key_new();
g_signal_connect(key_ctrl, "key-pressed", G_CALLBACK(on_key_press), lua_state);
gtk_widget_add_controller(drawing_area, key_ctrl);
```

### GTK4 렌더링 연결 (C++ 코드 패턴)
```cpp
// GTK3의 "draw" 시그널 금지 — set_draw_func 만 사용
gtk_drawing_area_set_draw_func(
    GTK_DRAWING_AREA(drawing_area),
    draw_callback,   // ← 이 안에서 lua_call("on_render") 호출
    lua_state,
    NULL
);
```

---

## 16. REQUIRED CORE COMPONENTS
The application MUST contain:
```text
1. Main Editor Canvas Renderer (GtkDrawingArea + Cairo 기반)
2. Text buffer                 (custom, NOT system widget)
3. Lua runtime                 (embedded lua_State)
4. Keyboard input handler      (GtkEventControllerKey & GtkIMContext 기반)
5. Viewport logic              (Lua 소유)
6. Cairo rendering system      (cairo_t drawing context 기반)
```

---

## 26. TEXT BUFFER MODEL
Preferred structure:
```cpp
std::vector<std::string> lines;
```
Encoding:
```text
UTF-8
```

---

## 27. FILE SIZE LIMIT
Maximum:
```text
10,000 lines
```
If exceeded:
```text
abort loading immediately
close file handle
show warning message
```

---

## 29. LUA RESPONSIBILITIES
Lua owns:
```text
cursor state
viewport state
editor logic
motion logic
render ordering
```

---

## 30. C++ RESPONSIBILITIES
C++ owns:
```text
GTK4 GDK event handling
GtkDrawingArea Cairo rendering
GtkIMContext input pipeline
file I/O
process launch (curl, LSP)
thread launch
```

---

## 34. EXTENSION POLICY
Extensions are OPTIONAL.
NOT core systems.
All extensions MUST be:
```text
failure-tolerant
removable
deactivatable
```
If any extension fails, the base editor MUST continue to operate.

---

## 44. REQUIRED COMPONENT COUNT POLICY
Target component count:
```text
5 ~ 10 core modules maximum
```
Avoid:
```text
placeholder explosion
generic modules
fake architecture inflation
```

---

## 45. SUCCESS CONDITIONS
Project success means:
```text
GTK4 Cairo canvas rendering success
+ Lua runtime connection success
+ custom text buffer success
+ external process attachment success
```
NOT:
```text
feature completeness
```

---

## 47. HARDCODED GIT DOWNLOADER TARGETS
비동기 curl 다운로더가 참조하는 공식 고정 URL 테이블.
포크(Fork) 버전 사용 금지. 아래 주소만 허용.
```json
{
  "git_downloader_targets": {
    "treesitter": "https://github.com/romus204/tree-sitter-manager.nvim",
    "lsp":        "https://github.com/neovim/nvim-lspconfig",
    "gruvbox":    "https://github.com/morhetz/gruvbox",
    "dracula":    "https://github.com/maxmx03/dracula.nvim",
    "nord":       "https://github.com/kunzaatko/nord.nvim",
    "catppuccin": "https://github.com/catppuccin/nvim"
  }
}
```

### 다운로더 동작 규칙
```text
[설치 / 업데이트 확인] 클릭 시:
  → 백그라운드 std::thread 생성
  → curl -L {url}/archive/refs/heads/main.zip \
         -o ~/.config/my_IDE/ext/{name}.zip
  → 압축 해제 후 ~/.config/my_IDE/ext/{name}/ 에 overwrite 설치

[설정] 클릭 시:
  → ~/.config/my_IDE/config/{name}_config.lua 를 에디터 버퍼에 직접 오픈
  → 유저가 텍스트로 직접 수정
```

### 경로 정책 (XDG 표준)
```text
기준 루트:  ~/.config/my_IDE/

설정 파일:  ~/.config/my_IDE/config/
  gruvbox_config.lua
  nord_config.lua
  dracula_config.lua
  catppuccin_config.lua
  lsp_config.lua

확장 설치:  ~/.config/my_IDE/ext/
  treesitter/
  lsp/
  themes/gruvbox/
  themes/dracula/
  themes/nord/
  themes/catppuccin/
```

### C++ 경로 해석 규칙
```cpp
// 시작 시 XDG_CONFIG_HOME 환경변수 우선 확인, 없으면 ~/.config 사용
const char* xdg = getenv("XDG_CONFIG_HOME");
std::string base = xdg ? std::string(xdg) : (getenv("HOME") + std::string("/.config"));
std::string config_root = base + "/my_IDE/";
```
경로에 공백이 포함될 경우 반드시 쌍따옴표로 감쌀 것:
```cpp
"\"" + install_path + "\""
```
> ⚠️ NOTE: Portable 완전 독립 실행은 포기. Linux XDG 표준 준수 우선.

---

## 50. PROJECT DIRECTORY STRUCTURE
AXON과 LLM은 아래 폴더 구조를 **반드시 그대로** 생성해야 한다.
임의로 디렉토리를 추가하거나 삭제하지 마라.

### 프로젝트 루트 구조
```text
MyLightIDE/                          ← 프로젝트 루트
├── CMakeLists.txt                   ← 빌드 명세 (GCC + GTK4 >= 4.6)
├── build/                           ← 빌드 출력 (git ignore 대상)
│
├── src/                             ← C++ 소스 파일
│   ├── main.cpp                     ← 진입점 / GtkApplication 초기화
│   ├── gtk_app.cpp                  ← GTK4 윈도우/위젯 레이아웃 구성
│   ├── text_buffer.cpp              ← 텍스트 버퍼 (std::vector<std::string>)
│   ├── lua_runtime.cpp              ← Lua 5.4 런타임 초기화 및 브릿지
│   ├── input.cpp                    ← GTK4 EventController 키입력 처리
│   └── terminal_renderer.cpp        ← GtkDrawingArea + Cairo 렌더링
│
├── include/                         ← C++ 헤더 파일
│   ├── gtk_app.h
│   ├── text_buffer.h
│   ├── lua_runtime.h
│   ├── input.h
│   └── terminal_renderer.h
│
└── lua/                             ← Lua 스크립트 (런타임 로직)
    ├── core/
    │   ├── editor.lua               ← 커서/뷰포트 상태 관리
    │   ├── renderer.lua             ← 렌더 순서 및 레이아웃 로직
    │   └── motion.lua               ← 커서 이동 로직
    └── init.lua                     ← Lua 런타임 진입점
```

### 각 소스 파일 책임 요약
| 파일 | 소유권 | 핵심 역할 |
|------|--------|-----------|
| `main.cpp` | C++ | `GtkApplication` 생성 및 `g_application_run()` |
| `gtk_app.cpp` | C++ | 위젯 레이아웃 구성, EventController 등록 |
| `text_buffer.cpp` | C++ | `std::vector<std::string>` 기반 버퍼 CRUD |
| `lua_runtime.cpp` | C++ | `lua_State` 초기화, C++ ↔ Lua 함수 바인딩 |
| `input.cpp` | C++ | `GtkEventControllerKey` → Lua `on_key` 포워딩 |
| `terminal_renderer.cpp` | C++ | Cairo `draw_callback` → Lua `on_render` 포워딩 |
| `editor.lua` | **Lua** | 커서 위치, 뷰포트 스크롤 상태 소유 |
| `renderer.lua` | **Lua** | 렌더 순서 결정, 라인 인덱싱 |
| `motion.lua` | **Lua** | h/j/k/l, w/b 커서 이동 매핑 |

### 금지 규칙
```text
src/libc.cpp        금지 — 시스템 라이브러리
src/gtk.cpp         금지 — 시스템 라이브러리
src/cairo.cpp       금지 — 시스템 라이브러리
src/pthread.cpp     금지 — 시스템 라이브러리
tabs/               금지 — 탭 기능 없음
split_view/         금지 — 분할 뷰 없음
marketplace/        금지 — 확장 마켓플레이스 없음
```

---

## 28. LUA RUNTIME CONTRACT

### Lua 버전 확정
```text
Lua 버전: 5.4 ONLY
이전 버전(5.3, 5.2, 5.1) 사용 금지
LuaJIT 사용 금지
```

### 링킹 방식
```text
방식: Static Linking (정적 링킹) 강제
이유: 배포 환경에서 Lua 공유 라이브러리 의존성 제거
```
CMake 정적 링킹 추가 예시:
```cmake
find_package(Lua 5.4 REQUIRED)
target_include_directories(MyLightIDE PRIVATE ${LUA_INCLUDE_DIR})
target_link_libraries(MyLightIDE ${LUA_LIBRARIES} pthread dl m)
```

### 프로세스당 lua_State 수
```text
lua_State: 프로세스당 1개
공유 상태(shared state) 금지
멀티 lua_State 금지
```

---

## 31. LUA INTEGRATION RULES

### 허용
```text
static linking         ← Lua 5.4 소스 직접 컴파일
embedded runtime       ← lua_State 단일 인스턴스
lua/*.lua 파일 로드    ← luaL_dofile() / luaL_loadfile()
```

### 금지
```text
LuaRocks               금지 — 패키지 매니저 의존성
runtime 패키지 설치    금지 — 실행 중 외부 모듈 설치
원격 스크립트 로드     금지 — require("http://...")
동적 lua_State 생성    금지 — 멀티 상태 구조
```

---

## 51. LUA RENDERER FLOW & C++ BINDING TABLE

### 렌더링 흐름 (전체 파이프라인)
```text
[GTK4 draw 이벤트]
        ↓
draw_callback() — C++ (terminal_renderer.cpp)
        ↓
lua_getglobal(L, "on_render")
lua_pushnumber(L, cairo_ctx_ptr)
lua_call(L, 1, 0)
        ↓
on_render(ctx) — Lua (renderer.lua)
        ↓
뷰포트 계산: editor.get_viewport_lines()
        ↓
반복: draw_text(x, y, line) 호출 (Lua → C++)
        ↓
Cairo: cairo_show_text() — C++ 네이티브 실행
```

### 뷰포트 계산 규칙
```text
렌더링 대상: 화면에 보이는 줄만 (최대 50줄)
계산 공식:
  visible_start = scroll_offset
  visible_end   = scroll_offset + floor(canvas_height / line_height)
  
전체 버퍼 렌더링 금지 — 반드시 viewport 범위만 처리
```

### C++ → Lua 노출 함수 (바인딩 테이블)
`lua_runtime.cpp`에서 Lua에 등록해야 할 C++ 네이티브 함수 목록:
| Lua 함수명 | C++ 구현 | 설명 |
|-----------|---------|------|
| `draw_text(x, y, text)` | `cairo_show_text()` | 지정 좌표에 텍스트 렌더링 |
| `set_color(r, g, b)` | `cairo_set_source_rgb()` | 렌더 색상 설정 |
| `set_font_size(size)` | `cairo_set_font_size()` | 폰트 크기 설정 |
| `get_canvas_width()` | GTK `gtk_widget_get_width()` | 캔버스 가로폭 반환 |
| `get_canvas_height()` | GTK `gtk_widget_get_height()` | 캔버스 세로폭 반환 |
| `get_line_height()` | Cairo `font_extents` | 줄 높이 계산값 반환 |
| `request_redraw()` | `gtk_widget_queue_draw()` | 다음 프레임 강제 재렌더 |
| `get_buffer_line(n)` | `text_buffer.get_line(n)` | n번째 줄 텍스트 반환 |
| `get_buffer_line_count()` | `text_buffer.line_count()` | 전체 줄 수 반환 |

### C++ 바인딩 등록 패턴
```cpp
static int l_draw_text(lua_State* L) {
    double x    = luaL_checknumber(L, 1);
    double y    = luaL_checknumber(L, 2);
    const char* text = luaL_checkstring(L, 3);
    cairo_move_to(g_cairo_ctx, x, y);
    cairo_show_text(g_cairo_ctx, text);
    return 0;
}

void lua_register_native_functions(lua_State* L) {
    lua_register(L, "draw_text",          l_draw_text);
    lua_register(L, "set_color",          l_set_color);
    lua_register(L, "set_font_size",      l_set_font_size);
    lua_register(L, "get_canvas_width",   l_get_canvas_width);
    lua_register(L, "get_canvas_height",  l_get_canvas_height);
    lua_register(L, "get_line_height",    l_get_line_height);
    lua_register(L, "request_redraw",     l_request_redraw);
    lua_register(L, "get_buffer_line",    l_get_buffer_line);
    lua_register(L, "get_buffer_line_count", l_get_buffer_line_count);
}
```

### Lua 렌더러 구현 예시 (renderer.lua)
```lua
function on_render()
    local total    = get_buffer_line_count()
    local height   = get_canvas_height()
    local lh       = get_line_height()
    local visible  = math.floor(height / lh)

    set_color(0.12, 0.12, 0.12)

    local start_line = editor.scroll_offset + 1
    local end_line   = math.min(start_line + visible - 1, total)

    for i = start_line, end_line do
        local y = (i - start_line) * lh + lh
        set_color(0.9, 0.9, 0.9)
        draw_text(8, y, get_buffer_line(i))
    end

    editor.draw_cursor()
end
```

---

## 52. LUA SCRIPT PATH RESOLUTION
Lua 스크립트 경로는 반드시 실행파일(ELF) 위치 기준으로 계산해야 한다. CWD 기준 상대경로는 사용 금지.

### 필수 구현 패턴 (C++)
```cpp
char exe_buf[4096] = {};
ssize_t len = readlink("/proc/self/exe", exe_buf, sizeof(exe_buf) - 1);
if (len > 0) {
    exe_buf[len] = '\0';
    auto exe_dir  = std::filesystem::path(exe_buf).parent_path();
    auto candidate = exe_dir / ".." / "lua";
    if (std::filesystem::exists(candidate))
        script_dir = std::filesystem::canonical(candidate).string();
}
```

### 빌드 요건 추가
```cmake
target_link_libraries(MyLightIDE ... stdc++fs)
```

---

## 53. PRINTABLE CHARACTER INPUT FLOW

### 입력 분류 원칙
GTK4 key-pressed 이벤트 수신 시:
[printable 문자] `gdk_keyval_to_unicode(keyval) >= 0x20`
    → C++ 직접 처리: `buffer->insert_char(line, col, ch)`
    → Lua `editor.cursor_col += 1`
    → `gtk_widget_queue_draw()`

[Backspace] `keyval == GDK_KEY_BackSpace`
    → C++: `buffer->delete_char(line, col-1)`
    → Lua `editor.cursor_col -= 1`

[Enter] `keyval == GDK_KEY_Return`
    → C++: `buffer->insert_newline(line, col)`
    → Lua `editor.cursor_line += 1, cursor_col = 0`

[네비게이션] 화살표, hjkl 등
    → Lua `on_key(keyval, modifiers)` 로 포워딩

### Modifier 필터링
```text
Ctrl / Alt 조합키가 눌린 경우 → printable 삽입 금지
```

---

## 54. FILE EXPLORER IMPLEMENTATION
구현 방식: `std::filesystem::directory_iterator`
위젯: `GtkListBox` + `GtkScrolledWindow`
파일 클릭: `open_file_in_editor()` → `buffer->load_file()` → 에디터 갱신

### 동작 규칙
1. 앱 시작 시 CWD의 파일 목록을 열거
2. 숨김 파일(. 으로 시작) 은 표시하지 않음
3. 아이콘 구분: 📂 디렉토리, 📄 일반 파일
4. 파일 클릭 시:
     → `buffer->load_file(path)`
     → Lua editor 커서/스크롤 초기화
     → status_label 에 파일 경로 표시
     → `gtk_widget_queue_draw()`
5. 디렉토리 클릭: 경고 출력만 (하위 탐색 미구현)

### 파일 로드 오류 처리
```text
load_file() 예외 발생 시:
  → g_warning() 출력
  → 에디터 버퍼 변경 없음
```

---

## 55. EXTENSION MANAGER IMPLEMENTATION
고정 확장 목록 (§47 URL 하드코딩)
```text
ID          표시명       GitHub URL
──────────────────────────────────────────────────────────────
treesitter  Tree-sitter  github.com/romus204/tree-sitter-manager.nvim
lsp         LSP          github.com/neovim/nvim-lspconfig
gruvbox     Gruvbox      github.com/morhetz/gruvbox
dracula     Dracula      github.com/maxmx03/dracula.nvim
nord        Nord         github.com/kunzaatko/nord.nvim
catppuccin  Catppuccin   github.com/catppuccin/nvim
```

### UI 구조 (각 확장 항목)
```text
GtkListBoxRow
  └─ GtkBox (vertical)
      ├─ GtkLabel  "표시명"
      ├─ GtkLabel  "github.com/..."
      └─ GtkBox (horizontal)
          ├─ GtkButton  "⬇ 설치/업데이트"
          └─ GtkButton  "⚙ 설정"
```

### [설치/업데이트] 버튼 동작
1. 버튼 레이블 → "⬇ 다운로드 중..."
2. 버튼 비활성화
3. `g_spawn_async()` 로 curl 비동기 실행:
     `curl -L {url}/archive/refs/heads/main.zip -o ~/.config/my_IDE/ext/{id}.zip`
4. spawn 성공: 레이블 → "✅ 완료"
   spawn 실패: 레이블 → "❌ 실패"
5. 버튼 재활성화

### [설정] 버튼 동작
1. `~/.config/my_IDE/config/{id}_config.lua` 파일 존재 확인
2. 없으면 기본 템플릿으로 자동 생성
3. `open_file_in_editor()` → 에디터 버퍼에 config 파일 오픈

---

## 56. IME (GtkIMContext / IBus) INPUT SPECIFICATION

### 1. GtkIMContext 도입 규칙
에디터 DrawingArea 위젯 생성 시 다음과 같이 IM Context를 설정 및 연결해야 한다.
```cpp
GtkIMContext* im_context = gtk_im_multicontext_new();
gtk_im_context_set_client_widget(im_context, GTK_WIDGET(drawing_area));
```

### 2. IM Context 신호 연동
* **`commit` 신호 (텍스트 완성)**:
  - IME 조합이 완성된 UTF-8 문자열이 커밋될 때 트리거되어 C++ 버퍼 및 Lua 커서 위치 동기화를 수행한다.
* **`preedit-changed` 신호 (조합 중 상태)**:
  - IME 조합 중인 중간 글자(preedit string)를 화면 상의 커서 위치에 임시 드로잉하도록 렌더러에 정보를 제공한다.
  - C++ `terminal_renderer.cpp`는 IM Context의 preedit 문자열을 읽어 현재 커서 위치 뒤에 밑줄 또는 반투명 배경으로 그린다.

### 3. IBus 포커스 제어 규칙
위젯(`drawing_area`)에 `GtkEventControllerFocus`를 추가하여 포커스 전환 시 IME 입력 윈도우 유실을 막는다.
```cpp
GtkEventController* focus_ctrl = gtk_event_controller_focus_new();
g_signal_connect(focus_ctrl, "enter", G_CALLBACK(+[](GtkEventControllerFocus*, gpointer data) {
    gtk_im_context_focus_in(GTK_IM_CONTEXT(data));
}), im_context);
g_signal_connect(focus_ctrl, "leave", G_CALLBACK(+[](GtkEventControllerFocus*, gpointer data) {
    gtk_im_context_focus_out(GTK_IM_CONTEXT(data));
}), im_context);
gtk_widget_add_controller(drawing_area, focus_ctrl);
```

### 4. 이벤트 필터링 흐름
키보드 이벤트 발생 시 **반드시** IM Context가 이벤트를 먼저 필터링하도록 보장한다.
```cpp
static gboolean on_key_press(GtkEventControllerKey* ctrl, guint keyval, guint keycode, GdkModifierType state, gpointer user_data) {
    GdkEvent* event = gtk_event_controller_get_current_event(GTK_EVENT_CONTROLLER(ctrl));
    if (gtk_im_context_filter_keypress(im_context, event)) {
        return TRUE; 
    }
    // ...
}
```

---

## 57. FILE LIFECYCLE & MENU (GAction / GMenu / GtkFileDialog) SPECIFICATION

### 1. GAction & 단축키 바인딩
* `app.new-file` (Ctrl + N)
* `app.open-file` (Ctrl + O)
* `app.save-file` (Ctrl + S)
* `app.save-as-file` (Ctrl + Shift + S)

### 2. GtkFileDialog 사용 규칙 (GTK4 표준)
비동기 호출 인터페이스인 `GtkFileDialog`만 사용한다.
`gtk_file_dialog_open()` 및 `gtk_file_dialog_save()` 인터페이스를 사용하고, GAsyncReadyCallback 비동기 콜백 패턴을 준수한다.

---

## 58. DYNAMIC FILE EXPLORER (Directory Navigation) SPECIFICATION

### 1. 디렉토리 탐색 상태 (C++ 소유)
파일 탐색기의 최상단에 항상 **`.. (상위 디렉토리로 이동)`** 항목을 유지한다. 사용자가 디렉토리 항목을 더블클릭하면 `current_dir` 값을 업데이트하고 목록을 동적으로 재조회한다.

### 2. 디렉토리 변경 시 UI 동기화
경로가 바뀔 때마다 탐색기 상단의 📁 헤더 라벨 텍스트를 현재 경로로 동적 갱신한다.

---

## 59. EXTENSION HARDENING (Installation, Unzip, Runtime Isolation) SPECIFICATION

### 1. 비동기 다운로드 및 압축 해제 파이프라인 (Unzip 연쇄)
`g_child_watch_add`를 사용하여 curl 프로세스가 반환코드 0으로 완전히 종료되었는지를 확인 후 `unzip -o {zip_path} -d ~/.config/my_IDE/ext/{id}/` 프로세스를 비동기 실행한다.

### 2. 강력한 확장 격리 로딩 (Extension Isolation Boundary)
Lua 코어 로딩 함수는 반드시 `luaL_dofile()` 대신 `lua_pcall()` 이나 `pcall(require, ...)` 구조를 이용하여 감싸 개별 스크립트 실행 중 에러가 발생해도 메인 프로세스가 중단되지 않도록 보호한다.

---

## 60. VISUAL MENU BAR SPECIFICATION (시각적 메뉴바 사양)
`GMenu` 구조를 생성하고 `GtkPopoverMenuBar` 위젯을 생성하여 최상단 vertical box의 가장 첫 번째 자식으로 삽입한다.

---

## 61. HANGUL/ENGLISH TOGGLE & IBUS HARDENING SPECIFICATION (한영 전환 및 IBus 안정화 사양)
한영 전환키(`GDK_KEY_Hangul`) 및 기능 키(`keyval >= 0xFF00`) 입력 시 C++ printable 문자 직접 삽입 분기에서 철저히 제외한다.

---

## 62. FILE EXPLORER SORTING & VISIBILITY SPECIFICATION (파일 탐색기 정렬 및 숨김 항목 노출 사양)
1. **상위 디렉토리 이동 (`..`)**: 항상 최상단 고정.
2. **폴더 그룹 (Directories)**: 그 뒤로 오름차순 알파벳 정렬 배치.
3. **파일 그룹 (Files)**: 대소문자 구분 없이 알파벳 정렬 배치. 숨김 파일(.`으로 시작)도 전부 노출한다.

---

## 63. EDITING CLIPBOARD SHORTCUTS SPECIFICATION (편집 단축키 및 클립보드 연동 사양)
* `Ctrl + V` (붙여넣기) 단축키 입력 시 `GdkClipboard` 비동기 텍스트 조회를 구동하고 버퍼에 반영한다.
* `Ctrl + C` / `Ctrl + X` (복사/잘라내기) 입력 시 텍스트를 `GdkClipboard`에 UTF-8 스트림으로 전송한다.
* `state & GDK_CONTROL_MASK` 조건이 성립하는 단축키 입력은 IME 필터링 함수를 통과하기 전에 먼저 차단 및 수신하여 C++ 복사/붙여넣기로 강제 라우팅한다.
