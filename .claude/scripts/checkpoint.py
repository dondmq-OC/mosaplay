#!/usr/bin/env python3
"""Save a checkpoint snapshot before context compression or session stop."""
import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent.parent
CHECKPOINT_DIR = WORKSPACE / ".claude" / "session_log" / "checkpoints"
MISSION_FILE = WORKSPACE / "tasks" / "mission.md"
EVENTS_FILE = WORKSPACE / ".claude" / "session_log" / "events.jsonl"

def main():
    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    snapshot = CHECKPOINT_DIR / f"checkpoint-{ts}.json"

    # Gather modified files from git
    try:
        result = subprocess.run(
            ["git", "status", "--porcelain"],
            capture_output=True, text=True, cwd=str(WORKSPACE)
        )
        modified = result.stdout.strip().split("\n") if result.stdout.strip() else []
    except Exception:
        modified = []

    data = {
        "timestamp": ts,
        "modified_files": modified,
        "mission_snapshot": "",
    }

    # Read current mission
    if MISSION_FILE.exists():
        data["mission_snapshot"] = MISSION_FILE.read_text()

    CHECKPOINT_DIR.mkdir(parents=True, exist_ok=True)
    snapshot.write_text(json.dumps(data, ensure_ascii=False, indent=2))

    # Log the checkpoint event
    event = {
        "ts": datetime.now(timezone.utc).isoformat(),
        "type": "checkpoint",
        "summary": f"Checkpoint saved: {snapshot.name}",
        "files": modified,
        "decision": "",
    }
    EVENTS_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(EVENTS_FILE, "a") as f:
        f.write(json.dumps(event, ensure_ascii=False) + "\n")

    print(f"[checkpoint] {snapshot.name} — {len(modified)} modified files")

if __name__ == "__main__":
    main()
