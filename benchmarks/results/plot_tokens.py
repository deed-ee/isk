"""Render the published within-ISK agent pilot medians (Python stdlib)."""
import json
from pathlib import Path
from statistics import median


ROOT = Path(__file__).resolve().parent
TASKS = [
    ("2026-11-22-controller", "controller-rename", "Controller rename"),
    ("2026-11-22-controller", "controller-repair", "Controller repair"),
    ("2026-11-22-parcel", "controller-parcel", "Parcel rename (with search)"),
]
LEFT, WIDTH, LIMIT = 190, 390, 3000
svg = ['<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 680 310" role="img" aria-labelledby="title desc">',
       '<title id="title">Median total tokens per agent session</title>',
       '<desc id="desc">Paired bars compare text tools with the same tools plus semantic operations. Five sessions per bar. Exact values appear on each bar.</desc>',
       '<rect width="680" height="310" fill="white"/>',
       '<g font-family="sans-serif" fill="#182d39">',
       '<text x="12" y="27" font-size="20">Median total tokens per session</text>',
       '<text x="12" y="50" font-size="13">Input + output · five sessions per bar · lower means fewer tokens</text>']
for i, (batch, task, title) in enumerate(TASKS):
    rows = [json.loads(line) for line in (ROOT / batch / "results.jsonl").read_text().splitlines()]
    top = 73 + i * 66
    svg.append(f'<text x="12" y="{top+21}" font-size="13">{title}</text>')
    for j, (arm, color) in enumerate([("text", "#405a82"), ("semantic", "#137565")]):
        values = [r["measurements"]["input_tokens"] + r["measurements"]["output_tokens"]
                  for r in rows if r["task"]["id"] == task and r["candidate"] == arm]
        assert len(values) == 5
        value = median(values)
        assert 0 <= value <= LIMIT
        w, y = WIDTH * value / LIMIT, top + 22*j
        svg += [f'<rect x="{LEFT}" y="{y}" width="{w:.2f}" height="17" fill="{color}"/>',
                f'<text x="{LEFT+w+6:.2f}" y="{y+14}" font-size="13">{value:,}</text>']
svg += [f'<path d="M{LEFT} 67v185" stroke="#677681"/>',
        '<rect x="12" y="275" width="15" height="15" fill="#405a82"/>',
        '<text x="33" y="288" font-size="14">Text tools</text>',
        '<rect x="154" y="275" width="15" height="15" fill="#137565"/>',
        '<text x="175" y="288" font-size="14">Same tools + semantic operations</text>', '</g></svg>']
(ROOT / "agent-pilot-tokens.svg").write_text("\n".join(svg) + "\n")
