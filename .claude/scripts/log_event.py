#!/usr/bin/env python3
"""Append an event to the session persistence log."""
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

LOG_FILE = Path(__file__).resolve().parent.parent / "session_log" / "events.jsonl"

def log_event(event_type: str, summary: str, files: list[str] | None = None, decision: str = ""):
    event = {
        "ts": datetime.now(timezone.utc).isoformat(),
        "type": event_type,
        "summary": summary,
        "files": files or [],
        "decision": decision,
    }
    LOG_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(LOG_FILE, "a") as f:
        f.write(json.dumps(event, ensure_ascii=False) + "\n")

if __name__ == "__main__":
    args = sys.argv[1:]
    if len(args) < 2:
        print("Usage: log_event.py <type> <summary> [files...] [--decision <text>]")
        sys.exit(1)
    event_type = args[0]
    summary = args[1]
    files = []
    decision = ""
    i = 2
    while i < len(args):
        if args[i] == "--decision" and i + 1 < len(args):
            decision = args[i + 1]
            i += 2
        else:
            files.append(args[i])
            i += 1
    log_event(event_type, summary, files, decision)
    print(f"[event] {event_type}: {summary}")
