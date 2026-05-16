
import sys

path = "/home/dogsinatas/rust_project/axon/crates/axon-daemon/src/server.rs"
with open(path, "r") as f:
    lines = f.readlines()

# Find start_server function
start_idx = -1
for i, line in enumerate(lines):
    if "pub async fn start_server" in line:
        start_idx = i
        break

if start_idx == -1:
    sys.exit(1)

# Find where the listener is bound (usually near the end of the function)
bind_idx = -1
for i in range(start_idx, len(lines)):
    if "std::net::TcpListener::bind(addr)?" in lines[i]:
        bind_idx = i
        break

if bind_idx != -1:
    # v0.0.30: Implement Port Hardening (SO_REUSEADDR) to ensure immediate resource recovery
    new_bind_logic = [
        "    // v0.0.30: [PORT_HARDENING] Ensure immediate port recovery on restart\n",
        "    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None)?;\n",
        "    socket.set_reuse_address(true)?;\n",
        "    #[cfg(not(windows))]\n",
        "    socket.set_reuse_port(true)?;\n",
        "    socket.bind(\u0026addr.into())?;\n",
        "    socket.listen(128)?;\n",
        "    socket.set_nonblocking(true)?;\n",
        "    let listener = tokio::net::TcpListener::from_std(socket.into())?;\n"
    ]
    
    # Replace the old bind/listen logic (usually 4-5 lines)
    # Find the end of old bind logic
    end_bind_idx = bind_idx
    for i in range(bind_idx, len(lines)):
        if "let listener =" in lines[i]:
            end_bind_idx = i + 1
            break
            
    lines[bind_idx:end_bind_idx] = new_bind_logic
    
    with open(path, "w") as f:
        f.writelines(lines)
    print("Successfully implemented Port Hardening (SO_REUSEADDR) in server.rs")
else:
    print("Could not find bind logic")
