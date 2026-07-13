#!/usr/bin/env python3
"""
FSOT Desktop Visual — live chew view + query I/O.

  http://127.0.0.1:3850

- Polls LIVE_STATUS / BODY_STATUS / telemetry
- POST /api/query  {"text":"..."}  → writes QUERY_IN.json (living TUI must be running)
- GET  /api/query_out              → latest QUERY_OUT.json
- GET  /api/health                → heartbeat (watchdog / keep-alive)

Auto-started by start_fsot_living.bat when possible.
Hardened: SO_REUSEADDR, exception-safe handlers, self-restart loop.
"""

from __future__ import annotations

import json
import mimetypes
import os
import socket
import sys
import time
import traceback
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[1]
STATE = ROOT.parent / "state" / "habitat-rust"
STATIC = Path(__file__).resolve().parent / "desktop_viz_static"
PORT = int(os.environ.get("FSOT_VIZ_PORT", "3850"))
HEARTBEAT_PATH = STATE / "VIZ_HEARTBEAT.json"
STARTED_AT = time.time()
REQ_COUNT = 0


def read_json(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None


def tail_lines(path: Path, n: int = 40):
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        return lines[-n:]
    except Exception:
        return []


def touch_heartbeat():
    try:
        STATE.mkdir(parents=True, exist_ok=True)
        payload = {
            "ok": True,
            "port": PORT,
            "pid": os.getpid(),
            "uptime_s": round(time.time() - STARTED_AT, 1),
            "requests": REQ_COUNT,
            "ts": time.time(),
            "state": str(STATE),
        }
        tmp = HEARTBEAT_PATH.with_suffix(".tmp")
        tmp.write_text(json.dumps(payload), encoding="utf-8")
        tmp.replace(HEARTBEAT_PATH)
    except Exception:
        pass


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *args):
        sys.stderr.write("[viz] " + (fmt % args) + "\n")
        sys.stderr.flush()

    def _cors(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.send_header("Cache-Control", "no-store")
        self.send_header("Connection", "keep-alive")

    def do_OPTIONS(self):
        self.send_response(204)
        self._cors()
        self.end_headers()

    def do_GET(self):
        global REQ_COUNT
        REQ_COUNT += 1
        if REQ_COUNT % 25 == 0:
            touch_heartbeat()

        u = urlparse(self.path)
        path = u.path

        try:
            if path in ("/", "/index.html"):
                return self._file(STATIC / "index.html", "text/html; charset=utf-8")
            if path.startswith("/static/"):
                rel = path[len("/static/") :]
                fp = STATIC / rel
                if not fp.is_file():
                    return self._404()
                ctype = mimetypes.guess_type(str(fp))[0] or "application/octet-stream"
                return self._file(fp, ctype)

            if path in ("/api/health", "/api/heartbeat", "/healthz"):
                touch_heartbeat()
                return self._json(
                    {
                        "ok": True,
                        "service": "fsot-desktop-viz",
                        "port": PORT,
                        "pid": os.getpid(),
                        "uptime_s": round(time.time() - STARTED_AT, 1),
                        "requests": REQ_COUNT,
                        "live": (STATE / "LIVE_STATUS.json").exists(),
                        "body": (STATE / "BODY_STATUS.json").exists(),
                    }
                )

            if path == "/api/live":
                return self._json(
                    read_json(STATE / "LIVE_STATUS.json")
                    or {"error": "no LIVE_STATUS — start living TUI"}
                )
            if path == "/api/body":
                return self._json(
                    read_json(STATE / "BODY_STATUS.json")
                    or {"error": "no BODY_STATUS — start mind_bridge"}
                )
            if path == "/api/activity":
                return self._json(read_json(STATE / "ACTIVITY.json") or {})
            if path == "/api/telemetry":
                return self._json({"lines": tail_lines(STATE / "BODY_TELEMETRY.jsonl", 50)})
            if path == "/api/query_out":
                return self._json(read_json(STATE / "QUERY_OUT.json") or {"waiting": True})
            if path == "/api/home":
                home = STATE / "home"
                genome = home / "genome"
                return self._json(
                    {
                        "genome_latest": (genome / "LATEST.txt").exists(),
                        "genome_champion": (genome / "CHAMPION.txt").exists(),
                        "genome_index": tail_lines(genome / "INDEX.jsonl", 8),
                        "identity": tail_lines(home / "identity" / "NOTES.txt", 6),
                        "knowledge_facts": tail_lines(home / "knowledge" / "facts.jsonl", 6),
                        "tools": tail_lines(home / "tools" / "skills.jsonl", 6),
                    }
                )
            if path == "/api/snapshot":
                live = read_json(STATE / "LIVE_STATUS.json") or {}
                body = read_json(STATE / "BODY_STATUS.json") or {}
                act = read_json(STATE / "ACTIVITY.json") or {}
                qout = read_json(STATE / "QUERY_OUT.json") or {}
                return self._json(
                    {
                        "live": live,
                        "body": body,
                        "activity": act,
                        "query_out": qout,
                        "telemetry_tail": tail_lines(STATE / "BODY_TELEMETRY.jsonl", 30),
                        "state_dir": str(STATE),
                        "mind_running": (STATE / "LIVE_STATUS.json").exists(),
                        "viz_uptime_s": round(time.time() - STARTED_AT, 1),
                        "viz_ok": True,
                    }
                )

            return self._404()
        except Exception as e:
            return self._json({"ok": False, "error": str(e)}, code=500)

    def do_POST(self):
        global REQ_COUNT
        REQ_COUNT += 1
        u = urlparse(self.path)
        if u.path != "/api/query":
            return self._404()
        try:
            n = int(self.headers.get("Content-Length", "0"))
            raw = self.rfile.read(n).decode("utf-8")
            data = json.loads(raw) if raw else {}
        except Exception as e:
            return self._json({"ok": False, "error": f"bad json: {e}"}, code=400)

        text = (data.get("text") or data.get("q") or "").strip()
        if not text:
            return self._json({"ok": False, "error": "empty text"}, code=400)

        STATE.mkdir(parents=True, exist_ok=True)
        qid = data.get("id") or f"viz-{uuid.uuid4().hex[:10]}"
        payload = {
            "id": qid,
            "text": text,
            "source": data.get("source") or "desktop_viz",
        }
        inbox = STATE / "QUERY_IN.json"
        if inbox.exists():
            return self._json(
                {
                    "ok": False,
                    "error": "query already pending — wait a moment",
                    "id": qid,
                },
                code=409,
            )
        tmp = STATE / "QUERY_IN.json.tmp"
        tmp.write_text(json.dumps(payload, indent=2), encoding="utf-8")
        tmp.replace(inbox)

        wait_ms = int(data.get("wait_ms") or 2500)
        deadline = time.time() + wait_ms / 1000.0
        out = None
        while time.time() < deadline:
            out = read_json(STATE / "QUERY_OUT.json")
            if out and out.get("id") == qid:
                break
            if out and out.get("text") == text and out.get("source") == "desktop_viz":
                break
            time.sleep(0.08)

        if out and (out.get("id") == qid or out.get("text") == text):
            return self._json({"ok": True, "id": qid, "result": out})
        return self._json(
            {
                "ok": True,
                "id": qid,
                "pending": True,
                "hint": "QUERY_IN written — ensure living TUI is running; poll /api/query_out",
            }
        )

    def _file(self, fp: Path, ctype: str):
        data = fp.read_bytes()
        self.send_response(200)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(data)))
        self._cors()
        self.end_headers()
        self.wfile.write(data)

    def _json(self, obj, code=200):
        data = json.dumps(obj).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self._cors()
        self.end_headers()
        self.wfile.write(data)

    def _404(self):
        self.send_response(404)
        self._cors()
        self.end_headers()
        self.wfile.write(b"not found")

    def handle_one_request(self):
        try:
            super().handle_one_request()
        except (ConnectionResetError, BrokenPipeError, TimeoutError, OSError):
            pass  # client hangup — keep server alive
        except Exception:
            traceback.print_exc(file=sys.stderr)


class ReuseHTTPServer(ThreadingHTTPServer):
    allow_reuse_address = True
    daemon_threads = True

    def server_bind(self):
        self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        super().server_bind()


def serve_once():
    STATIC.mkdir(parents=True, exist_ok=True)
    STATE.mkdir(parents=True, exist_ok=True)
    print("FSOT Desktop Visual + Query I/O")
    print(f"  state: {STATE}")
    print(f"  open:  http://127.0.0.1:{PORT}")
    print(f"  health: http://127.0.0.1:{PORT}/api/health")
    print("  Type in the browser chat box (living TUI must be running to answer).")
    sys.stdout.flush()
    httpd = ReuseHTTPServer(("127.0.0.1", PORT), Handler)
    httpd.timeout = 1.0
    touch_heartbeat()
    try:
        httpd.serve_forever(poll_interval=0.5)
    except KeyboardInterrupt:
        print("\nbye")
        return 0
    finally:
        try:
            httpd.server_close()
        except Exception:
            pass
    return 1


def main():
    # Self-restart loop so a single crash doesn't kill the desktop visual.
    # Set FSOT_VIZ_NO_RESTART=1 to disable.
    restart = os.environ.get("FSOT_VIZ_NO_RESTART", "").lower() not in ("1", "true", "yes")
    backoff = 0.5
    while True:
        try:
            code = serve_once()
            if code == 0 or not restart:
                return
        except OSError as e:
            # Port in use — report and wait
            print(f"[viz] bind error: {e} — retry in {backoff:.1f}s", file=sys.stderr)
            sys.stderr.flush()
        except Exception:
            traceback.print_exc(file=sys.stderr)
            sys.stderr.flush()
        if not restart:
            break
        time.sleep(backoff)
        backoff = min(backoff * 1.5, 8.0)
        print("[viz] restarting serve loop…", flush=True)


if __name__ == "__main__":
    main()
