#!/usr/bin/env python3
"""Render query_chan_basic JSON as a standalone visual-check HTML file.

Draws:
- raw K-lines
- include-processed merged boxes
- FX points and FX connection line
- BI lines when available
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True, help="JSON file produced by query_chan_basic")
    parser.add_argument("--out", required=True, help="output HTML file")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    data = json.loads(Path(args.input).read_text(encoding="utf-8"))
    html = build_html(data)
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(html, encoding="utf-8")
    print(f"wrote {out}")
    return 0


def build_html(data: dict) -> str:
    payload = json.dumps(data, ensure_ascii=False)
    return f"""<!doctype html>
<html lang="zh-CN">
<head>
<meta charset="utf-8" />
<title>Chan6 Basic Visual Check</title>
<style>
  html, body {{ margin: 0; padding: 0; background: #0b0f14; color: #d6e2f0; font-family: Consolas, 'Microsoft YaHei', monospace; }}
  #toolbar {{ padding: 10px 14px; border-bottom: 1px solid #263241; display: flex; gap: 18px; align-items: center; flex-wrap: wrap; }}
  #chart {{ display: block; width: 100vw; height: calc(100vh - 54px); }}
  label {{ user-select: none; }}
  input {{ vertical-align: middle; }}
  .hint {{ color: #8aa0b8; }}
</style>
</head>
<body>
<div id="toolbar">
  <strong>Chan6 Basic Visual Check</strong>
  <label><input id="showMerged" type="checkbox" checked /> 合并框</label>
  <label><input id="showFx" type="checkbox" checked /> 分型点/连线</label>
  <label><input id="showBi" type="checkbox" checked /> 笔线</label>
  <span id="stats" class="hint"></span>
</div>
<canvas id="chart"></canvas>
<script>
const DATA = {payload};
const canvas = document.getElementById('chart');
const ctx = canvas.getContext('2d');
const showMerged = document.getElementById('showMerged');
const showFx = document.getElementById('showFx');
const showBi = document.getElementById('showBi');
const stats = document.getElementById('stats');

function resize() {{
  const dpr = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  canvas.width = Math.max(1, Math.floor(rect.width * dpr));
  canvas.height = Math.max(1, Math.floor(rect.height * dpr));
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  draw();
}}

function priceBounds() {{
  const kline = DATA.kline || [];
  const boxes = DATA.merged_boxes || [];
  let hi = -Infinity, lo = Infinity;
  for (const k of kline) {{ hi = Math.max(hi, k.high); lo = Math.min(lo, k.low); }}
  for (const b of boxes) {{ hi = Math.max(hi, b.high); lo = Math.min(lo, b.low); }}
  if (!Number.isFinite(hi) || !Number.isFinite(lo) || hi === lo) {{ hi = 1; lo = 0; }}
  const pad = (hi - lo) * 0.08;
  return {{ hi: hi + pad, lo: lo - pad }};
}}

function makeScale() {{
  const kline = DATA.kline || [];
  const rect = canvas.getBoundingClientRect();
  const left = 56, right = 24, top = 20, bottom = 34;
  const w = Math.max(1, rect.width - left - right);
  const h = Math.max(1, rect.height - top - bottom);
  const barWidth = kline.length > 0 ? Math.max(3, w / kline.length) : 8;
  const byId = new Map();
  kline.forEach((k, i) => byId.set(k.bar_id, i));
  const bounds = priceBounds();
  function xByIndex(i) {{ return left + i * barWidth + barWidth / 2; }}
  function xByBarId(barId) {{
    if (byId.has(barId)) return xByIndex(byId.get(barId));
    if (kline.length === 0) return left;
    const first = kline[0].bar_id;
    return xByIndex(barId - first);
  }}
  function y(price) {{ return top + (bounds.hi - price) / (bounds.hi - bounds.lo) * h; }}
  return {{ left, right, top, bottom, w, h, barWidth, byId, xByIndex, xByBarId, y, bounds }};
}}

function drawGrid(scale) {{
  const rect = canvas.getBoundingClientRect();
  ctx.clearRect(0, 0, rect.width, rect.height);
  ctx.fillStyle = '#0b0f14';
  ctx.fillRect(0, 0, rect.width, rect.height);
  ctx.strokeStyle = '#1e2a37';
  ctx.lineWidth = 1;
  ctx.font = '12px Consolas, monospace';
  ctx.fillStyle = '#74869a';
  for (let i = 0; i <= 5; i++) {{
    const y = scale.top + scale.h * i / 5;
    const p = scale.bounds.hi - (scale.bounds.hi - scale.bounds.lo) * i / 5;
    ctx.beginPath(); ctx.moveTo(scale.left, y); ctx.lineTo(rect.width - scale.right, y); ctx.stroke();
    ctx.fillText(p.toFixed(2), 6, y + 4);
  }}
}}

function drawKlines(scale) {{
  const kline = DATA.kline || [];
  const bodyW = Math.max(2, scale.barWidth * 0.55);
  for (let i = 0; i < kline.length; i++) {{
    const k = kline[i];
    const x = scale.xByIndex(i);
    const yHigh = scale.y(k.high), yLow = scale.y(k.low), yOpen = scale.y(k.open), yClose = scale.y(k.close);
    const up = k.close >= k.open;
    ctx.strokeStyle = up ? '#d95757' : '#38a169';
    ctx.fillStyle = up ? 'rgba(217,87,87,0.62)' : 'rgba(56,161,105,0.62)';
    ctx.lineWidth = 1;
    ctx.beginPath(); ctx.moveTo(x, yHigh); ctx.lineTo(x, yLow); ctx.stroke();
    const top = Math.min(yOpen, yClose);
    const height = Math.max(1, Math.abs(yClose - yOpen));
    ctx.fillRect(x - bodyW / 2, top, bodyW, height);
  }}
}}

function drawMergedBoxes(scale) {{
  const boxes = DATA.merged_boxes || [];
  for (const b of boxes) {{
    const x1 = scale.xByBarId(b.start_bar_id) - scale.barWidth * 0.48;
    const x2 = scale.xByBarId(b.end_bar_id) + scale.barWidth * 0.48;
    const y1 = scale.y(b.high);
    const y2 = scale.y(b.low);
    ctx.lineWidth = b.is_merged ? 1.8 : 0.8;
    ctx.strokeStyle = b.is_merged ? 'rgba(241,196,15,0.92)' : 'rgba(241,196,15,0.22)';
    ctx.fillStyle = b.is_merged ? 'rgba(241,196,15,0.10)' : 'rgba(241,196,15,0.025)';
    ctx.beginPath();
    ctx.rect(x1, y1, x2 - x1, y2 - y1);
    ctx.fill();
    ctx.stroke();
  }}
}}

function drawFx(scale) {{
  const fx = DATA.fx_lines || [];
  if (fx.length === 0) return;
  ctx.lineWidth = 1.5;
  ctx.strokeStyle = 'rgba(74,144,226,0.95)';
  ctx.beginPath();
  fx.forEach((f, i) => {{
    const x = scale.xByBarId(f.bar_id);
    const y = scale.y(f.price);
    if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
  }});
  ctx.stroke();
  for (const f of fx) {{
    const x = scale.xByBarId(f.bar_id);
    const y = scale.y(f.price);
    ctx.fillStyle = f.kind === 'top' ? '#ffcc66' : '#66d9ef';
    ctx.beginPath(); ctx.arc(x, y, 4, 0, Math.PI * 2); ctx.fill();
    ctx.fillStyle = '#cbd5e1';
    ctx.fillText(`${{f.kind}}#${{f.index}}`, x + 5, y - 5);
  }}
}}

function drawBi(scale) {{
  const bis = DATA.bi_lines || [];
  for (const bi of bis) {{
    ctx.lineWidth = 2.2;
    ctx.strokeStyle = bi.direction === 'up' ? '#ff6b6b' : '#4ade80';
    ctx.beginPath();
    ctx.moveTo(scale.xByBarId(bi.start_bar_id), scale.y(bi.start_price));
    ctx.lineTo(scale.xByBarId(bi.end_bar_id), scale.y(bi.end_price));
    ctx.stroke();
  }}
}}

function draw() {{
  const scale = makeScale();
  drawGrid(scale);
  drawKlines(scale);
  if (showMerged.checked) drawMergedBoxes(scale);
  if (showFx.checked) drawFx(scale);
  if (showBi.checked) drawBi(scale);
  const m = DATA.meta || {{}};
  stats.textContent = `kline=${{m.kline_count ?? 0}} merged=${{m.merged_count ?? 0}} fx=${{m.fx_count ?? 0}} bi=${{m.bi_count ?? 0}}`;
}}

window.addEventListener('resize', resize);
showMerged.addEventListener('change', draw);
showFx.addEventListener('change', draw);
showBi.addEventListener('change', draw);
resize();
</script>
</body>
</html>
"""


if __name__ == "__main__":
    raise SystemExit(main())
