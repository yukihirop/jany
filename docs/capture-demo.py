#!/usr/bin/env python3
"""docs/demo-capture.json を作る。jany を pty(stderr)+pipe(stdout)で実際に動かし、
[{"cmd": "jany …", "err": "<ANSI 付き stderr>", "line": "<stdout の 1 行>"}] を書く。
使い方: (logs/ に古い .log、app.tar.gz、dist/ を置いたディレクトリで) python3 docs/capture-demo.py > docs/demo-capture.json   (OPENROUTER_API_KEY が要る)
"""
import json, os, pty, subprocess, sys

SCENES = [
    ["find", "log", "files", "older", "than", "7", "days", "in", "logs", "delete", "--explain"],
    ["curl", "psot", "localhsot", "3000", "users", "first_name", "amanda"],
    ["docker", "run", "nginx", "8080:80", "background", "named", "web"],
    ["tar", "extrct", "app.tar.gz", "into", "dist", "strip", "1"],
]

def run(args):
    m, s = pty.openpty()
    os.environ.setdefault("COLUMNS", "100")
    p = subprocess.Popen(["jany", *args], stdout=subprocess.PIPE, stderr=s, close_fds=True)
    os.close(s)
    err = b""
    while True:
        try:
            chunk = os.read(m, 4096)
        except OSError:
            break
        if not chunk:
            break
        err += chunk
    out = p.stdout.read()
    p.wait()
    os.close(m)
    return err.decode(errors="replace"), out.decode(errors="replace").strip()

scenes = []
for args in SCENES:
    err, line = run(args)
    scenes.append({"cmd": "jany " + " ".join(args), "err": err, "line": line})
    print(f"{args[0]}: {len(err)} bytes stderr, line={line!r}", file=sys.stderr)
json.dump(scenes, sys.stdout, ensure_ascii=False, indent=1)
