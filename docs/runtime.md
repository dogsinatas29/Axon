# AXON Runtime

## Daemon

```bash
cargo build --release
./target/release/axon-daemon run    # Interactive mode — loads spec from ./spec/
```

- Bootstrap sequence: spec → IR → agent workspaces
- Spec loading from `./spec/` directory

## WorkBoard

- Thread-based task management
- Boss interrupt system (`[BOSS]` authority)
- Lock-in protocol for final approval

## Studio UI

| Endpoint | Description |
|----------|-------------|
| `http://localhost:8080` | Web viewer (Dashboard) |
| `http://localhost:5173` | React dev server (`cd studio && npm run dev`) |

## Hooks

- `tools/git-hooks/pre-push` — governance hook
- CI integration support

## Personas (Secondary)

- **Senior ([SNR])** — Code review, lock-in proposals, junior management
- **Junior ([JNR-N])** — Pure implementation, lounge participation

## Lounge System (Secondary)

- Auto-retrospective after task submission
- Intelligent participation based on interest weight
- Workaholic mode (1/10 weight during active tasks)

## HW/SW SPEC

| Component | Spec |
|-----------|------|
| CPU | Intel i7-4790 (8) @ 4.00 GHz |
| GPU | NVIDIA GTX 1050 Ti (4GB) |
| RAM | 16GB DDR3 |
| OS | Ubuntu 25.10 x86_64 |
| Kernel | Linux 6.18.6 |
