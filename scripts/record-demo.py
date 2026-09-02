#!/usr/bin/env python3
"""Record docs/demo.mp4: product lab + Playwright walk of the console."""

from __future__ import annotations

import atexit
import os
import shutil
import signal
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CLIENT = Path.home() / "connect-client"
LAB_PY = CLIENT / "scripts" / "lab.py"
OUT = ROOT / "docs" / "demo.mp4"
VIDEO_DIR = ROOT / "target" / "demo"
E2E = ROOT / "e2e"

CHILDREN: list[subprocess.Popen] = []


def die(msg: str, code: int = 1) -> None:
    print(msg, file=sys.stderr)
    sys.exit(code)


def cleanup() -> None:
    for p in reversed(CHILDREN):
        if p.poll() is None:
            try:
                os.killpg(p.pid, signal.SIGTERM)
            except OSError:
                p.terminate()
    time.sleep(0.3)
    for p in reversed(CHILDREN):
        if p.poll() is None:
            try:
                os.killpg(p.pid, signal.SIGKILL)
            except OSError:
                p.kill()


def wait_http(url: str, tries: int = 240) -> None:
    for _ in range(tries):
        try:
            with urllib.request.urlopen(url, timeout=0.5) as r:
                if r.status in (200, 204):
                    return
        except OSError:
            time.sleep(0.5)
    die(f"{url} did not become ready")


def ffmpeg_bin() -> str:
    cand = os.environ.get("FF") or str(Path.home() / ".local/bin/ffmpeg")
    if os.access(cand, os.X_OK):
        return cand
    found = shutil.which("ffmpeg") or ""
    if not found:
        die("ffmpeg not found")
    return found


def main() -> None:
    keep_lab = os.environ.get("KEEP_LAB") == "1"
    if not keep_lab:
        atexit.register(cleanup)
    started_lab = False
    os.environ.setdefault("DISPLAY", ":0.0")
    os.environ.setdefault("XAUTHORITY", str(Path.home() / ".Xauthority"))
    os.environ.setdefault(
        "DATABASE_URL", "postgres://postgres:postgres@127.0.0.1:5432/connect_lab"
    )
    if not LAB_PY.is_file():
        die(f"missing {LAB_PY}")
    playwright = E2E / "node_modules" / "playwright"
    if not playwright.is_dir():
        die(f"missing {playwright} — npm install in e2e/")

    (ROOT / "docs").mkdir(parents=True, exist_ok=True)
    if VIDEO_DIR.exists():
        shutil.rmtree(VIDEO_DIR)
    VIDEO_DIR.mkdir(parents=True)

    lab_up = False
    try:
        with urllib.request.urlopen("http://127.0.0.1:3040/api/health", timeout=0.5) as r:
            lab_up = r.status in (200, 204)
    except OSError:
        pass

    if not lab_up:
        print("lab…")
        lab = subprocess.Popen(
            [sys.executable, str(LAB_PY)],
            start_new_session=True,
            cwd=str(CLIENT),
        )
        CHILDREN.append(lab)
        started_lab = True
        wait_http("http://127.0.0.1:3040/api/health")

    print("drive…")
    env = {
        **os.environ,
        "CONSOLE_URL": "http://127.0.0.1:3040",
        "VIDEO_DIR": str(VIDEO_DIR),
        "HEADLESS": os.environ.get("HEADLESS", "0"),
    }
    subprocess.check_call(
        ["node", str(E2E / "demo-drive.mjs")],
        cwd=str(E2E),
        env=env,
    )
    webm = VIDEO_DIR / "raw.webm"
    if not webm.is_file():
        die("no video from playwright")
    ff = ffmpeg_bin()
    subprocess.check_call(
        [
            ff,
            "-y",
            "-i",
            str(webm),
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-crf",
            "18",
            "-preset",
            "fast",
            "-movflags",
            "+faststart",
            str(OUT),
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )
    print(OUT, OUT.stat().st_size)
    print(f"wrote {OUT}")
    if started_lab:
        cleanup()
        CHILDREN.clear()


if __name__ == "__main__":
    main()
