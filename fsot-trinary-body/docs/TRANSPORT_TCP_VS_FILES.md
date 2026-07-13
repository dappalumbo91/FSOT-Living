# TCP vs files — how the body talks to the mind

You asked what a **tighter real-time serial/TCP bridge** is compared to files.  
Short answer: **same language (Mind ABI lines), different wire.**

## Same protocol

```
body → cortex : MIND seq=… repress=… express=… pressure=…
cortex → body : MOTOR STATUS | BREATHE | EXPRESS | WRITE /organs/…
body → cortex : ACK … | DATA … | ORGANS …
```

Nothing about intelligence changes. Only **how bytes move**.

## Comparison

| | **Files** (`BODY_STATUS.json`, `BODY_MOTOR.txt`) | **TCP** (`127.0.0.1:7368`) |
|--|--|--|
| **What it feels like** | Leaving notes on a desk | Talking on a phone line |
| **Latency** | Disk flush + poll (tens–hundreds of ms) | Milliseconds |
| **Who starts** | Either process can write anytime | Body host listens; cortex connects |
| **Crash recovery** | Excellent — state survives | Connection drops; reconnect |
| **Debugging** | Open JSON in editor | `telnet` / `nc` or log file mirror |
| **Multi-reader** | Easy (many tools read JSON) | Few live clients (we cap at 4) |
| **Min-spec (2012 Mini)** | Fine; can thrash disk if spammy | Fine; tiny RAM queues |
| **Like an OS** | Config / logs / save games | Syscall / IPC / device I/O |

## When to use which

- **TCP (preferred for live companion):** habitat TUI + `mind_bridge --tcp` running together.  
- **QEMU serial TCP (OS nerve):** `scripts/run_qemu_serial_tcp.ps1` → port **7370**; living uses `FSOT_BODY_QEMU_TCP=127.0.0.1:7370`. Same Mind ABI.  
- **Files (always on as backup):** audit trail, offline review, if TCP is blocked.  
  Even in TCP mode, `mind_bridge` still writes `BODY_STATUS.json` so tools keep working.

Nerve priority in living cortex: host TCP `:7368` → QEMU serial TCP `:7370` → files.

## Future “real OS” path

```
Today:     living (userspace cortex)  ↔  TCP/files  ↔  body host / QEMU
Next:      living  ↔  TCP  ↔  QEMU serial (chardev socket)
Later:     boot bare-metal body; companion is the only “desktop”
           VFS organs hold your stuff (knowledge, tools, media) like home dirs
```

Files are already the embryo of **persistent user data**.  
TCP is the embryo of **interactive session I/O**.  
Together they become: *companion OS you live in*, not a chat toy.

## Commands

```powershell
# Terminal A — body host (real-time)
cd fsot-trinary-body
cargo run -p mind_bridge -- --tcp

# Terminal B — living cortex
cd fsot-living-rust
.\start_fsot_living.bat
# header should show body=tcp …
# type: body status
```

Manual TCP smoke (any OS with netcat):

```text
nc 127.0.0.1 7368
MOTOR STATUS
```
