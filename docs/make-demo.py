#!/usr/bin/env python3
"""docs/demo.svg を作る。入力は docs/capture-demo.py が取った実際の jany 出力の JSON:
[{"cmd": "jany ...", "err": "<stderr, ANSI 付き>", "line": "<stdout の 1 行>"}, ...]
stderr を流したあと、stdout の 1 行を「次のプロンプトに入力済み」の形($ … とカーソル)で見せる。
使い方: python3 docs/make-demo.py docs/demo-capture.json > docs/demo.svg
確認用: python3 docs/make-demo.py docs/demo-capture.json 12.5 > frame.svg  (12.5 秒時点の静止画)
"""
import json, re, sys, html

COLS, ROWS = 108, 23
CW, LH = 8.4, 19          # 文字幅・行高(px)。font-size 14 の等幅フォント前提
PAD_X, PAD_TOP, PAD_BOT = 20, 44, 16
W = int(PAD_X * 2 + COLS * CW)
H = int(PAD_TOP + ROWS * LH + PAD_BOT)

FG = "#e6eae8"
PALETTE = {30: "#3b4441", 31: "#f07178", 32: "#8fd9be", 33: "#f0c96a", 34: "#7aa2f7", 35: "#d3a4f7", 36: "#7ed6d4", 37: FG,
           90: "#66706e", 91: "#f07178", 92: "#8fd9be", 93: "#f0c96a", 94: "#7aa2f7", 95: "#d3a4f7", 96: "#7ed6d4", 97: FG}
DIM = "#97a19e"

SGR = re.compile(r"\x1b\[([0-9;]*)m")
OTHER = re.compile(r"\x1b\[[0-9;?]*[A-LN-Za-ln-z]")  # SGR(…m)以外のエスケープ


def ansi_to_spans(line):
    """1 行の ANSI 文字列 → [(text, fill, bold, reverse)]"""
    spans, fill, bold, dim, rev = [], FG, False, False, False
    pos = 0
    for m in SGR.finditer(line):
        if m.start() > pos:
            spans.append((line[pos:m.start()], DIM if dim else fill, bold, rev))
        for code in (m.group(1) or "0").split(";"):
            c = int(code or 0)
            if c == 0:
                fill, bold, dim, rev = FG, False, False, False
            elif c == 1:
                bold = True
            elif c == 2:
                dim = True
            elif c == 7:
                rev = True
            elif c == 22:
                bold = dim = False
            elif c == 27:
                rev = False
            elif c in PALETTE:
                fill = PALETTE[c]
        pos = m.end()
    if pos < len(line):
        spans.append((line[pos:], DIM if dim else fill, bold, rev))
    return spans


def clip(spans, n):
    """表示幅 n 文字で切って末尾に …"""
    out, used = [], 0
    for t, fill, bold, rev in spans:
        if used + len(t) > n:
            out.append((t[: max(0, n - 1 - used)] + "…", fill, bold, rev))
            return out
        out.append((t, fill, bold, rev))
        used += len(t)
    return out


def text_el(x, y, spans, cls, extra=""):
    inner = ""
    for t, fill, bold, rev in spans:
        if not t:
            continue
        style = f' fill="{fill}"' + (' font-weight="700"' if bold else "")
        inner += f"<tspan{style}>{html.escape(t)}</tspan>"
    return f'<text x="{x}" y="{y}" class="{cls}"{extra} xml:space="preserve">{inner}</text>'


def main(path, at=None):
    scenes = json.load(open(path))
    TYPE = 0.045        # 1 文字
    els, keyframes = [], []
    t = 0.8             # 現在時刻(s)
    for si, sc in enumerate(scenes):
        start = t
        rows = 0
        # コマンドを 1 文字ずつ
        cmd = sc["cmd"]
        y = PAD_TOP + LH * rows
        els.append(f'<g class="s{si}" data-scene="{si}">')
        els.append(f'<text x="{PAD_X}" y="{y}" fill="{DIM}" xml:space="preserve">$ </text>')
        for i, ch in enumerate(cmd):
            t += TYPE
            kf = f"k{si}_{len(els)}"
            keyframes.append((kf, t))
            x = PAD_X + CW * (2 + i)
            els.append(f'<text x="{x:.1f}" y="{y}" class="a" style="animation-name:{kf}" fill="{FG}" xml:space="preserve">{html.escape(ch)}</text>')
        t += 0.5
        rows += 1
        out = OTHER.sub("", sc["err"]).replace("\r", "")
        lines = out.split("\n")
        while lines and lines[-1].strip() == "":
            lines.pop()
        # jev を待つ場面は表が出るまで少し空く
        t += 0.7 if "jev:" in out else 0.35
        for ln in lines:
            if rows >= ROWS - 2:
                break
            plain = SGR.sub("", ln)
            y = PAD_TOP + LH * rows
            kf = f"k{si}_{len(els)}"
            keyframes.append((kf, t))
            els.append(text_el(PAD_X, y, clip(ansi_to_spans(ln), COLS), "a", f' style="animation-name:{kf}"'))
            rows += 1
            t += 0.25 if plain.startswith("$ ") else 0.03   # preview の実行は少し待つ
        # 次のプロンプトに、組み立てた 1 行が入力済みで現れる
        t += 0.6
        rows += 1
        y = PAD_TOP + LH * rows
        kf = f"k{si}_{len(els)}"
        keyframes.append((kf, t))
        line = sc["line"]
        spans = [("$ ", DIM, False, False), (line, FG, True, False)]
        els.append(text_el(PAD_X, y, clip(spans, COLS - 1), "a", f' style="animation-name:{kf}"'))
        cx = PAD_X + CW * min(len(line) + 2, COLS - 1) + 3
        els.append(f'<rect x="{cx:.1f}" y="{y - 13}" width="{CW:.1f}" height="17" fill="{FG}" class="a" style="animation-name:{kf},cursor;animation-duration:{"%.2f"},1s;animation-timing-function:step-end,step-end;animation-iteration-count:infinite,infinite"/>')
        t += 3.6              # 読む時間
        els.append("</g>")
        keyframes.append((f"scene{si}", (start, t)))
    T = t + 0.3
    els = [e.replace('animation-duration:%.2f,1s', f'animation-duration:{T:.2f}s,1s') for e in els]
    if at is not None:
        show = {name: v for name, v in keyframes}
        vis, cur = [], None
        for e in els:
            m = re.match(r'<g class="s(\d+)"', e)
            if m:
                cur = int(m.group(1))
                continue
            if e == "</g>":
                continue
            s0, s1 = show[f"scene{cur}"]
            if not (s0 <= at < s1):
                continue
            k = re.search(r"animation-name:(k\d+_\d+)", e)
            if k and show[k.group(1)] > at:
                continue
            vis.append(re.sub(r' class="a"| style="animation-[^"]*"', "", e))
        els = vis
    css = [
        f"@keyframes cursor{{0%,49%{{opacity:1}}50%,100%{{opacity:0}}}}",
        ".a{opacity:0;animation-duration:%.2fs;animation-timing-function:step-end;animation-iteration-count:infinite;animation-fill-mode:both}" % T,
    ]
    scene_css = []
    for name, v in keyframes:
        if name.startswith("scene"):
            s0, s1 = v
            p0, p1 = s0 / T * 100, s1 / T * 100
            scene_css.append(f"@keyframes {name}{{0%{{opacity:0}}{p0:.3f}%{{opacity:1}}{p1:.3f}%{{opacity:0}}100%{{opacity:0}}}}")
        else:
            p = v / T * 100
            css.append(f"@keyframes {name}{{0%{{opacity:0}}{p:.3f}%{{opacity:1}}100%{{opacity:1}}}}")
    for si in range(len(scenes)):
        css.append(f".s{si}{{animation:scene{si} {T:.2f}s step-end infinite}}")
    css += scene_css

    out = []
    out.append(f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="ui-monospace, SFMono-Regular, Menlo, Consolas, \'Liberation Mono\', monospace" font-size="14">')
    if at is None:
        out.append("<style>" + "".join(css) + "</style>")
    out.append(f'<rect width="{W}" height="{H}" rx="12" fill="#0f1716"/>')
    out.append(f'<rect width="{W}" height="32" rx="12" fill="#1a2423"/><rect y="20" width="{W}" height="12" fill="#1a2423"/>')
    for i, c in enumerate(["#ff5f57", "#febc2e", "#28c840"]):
        out.append(f'<circle cx="{20 + i * 20}" cy="16" r="6" fill="{c}"/>')
    out.append(f'<text x="{W / 2}" y="21" text-anchor="middle" font-size="12" fill="{DIM}">jany — jev × any command</text>')
    out.extend(els)
    out.append("</svg>")
    sys.stdout.write("\n".join(out))


main(sys.argv[1], float(sys.argv[2]) if len(sys.argv) > 2 else None)
