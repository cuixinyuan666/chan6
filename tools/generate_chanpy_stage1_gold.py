#!/usr/bin/env python3
"""Generate chan.py gold output for Chan6 stage-1 Rust alignment.

This script does not implement Chan logic. It calls the hichan Python backend
(`backend/app/chanpy_engine.py::analyze_bars`) and writes a compact gold JSON
fixture containing only the stage-1 structures used by Rust tests.
"""

from __future__ import annotations

import argparse
import csv
import json
import os
import sys
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--hichan-repo",
        default=os.environ.get("HICHAN_REPO"),
        help="Path to cuixinyuan666/chan_replay_app checked out at branch hichan. Defaults to HICHAN_REPO.",
    )
    parser.add_argument(
        "--input",
        default="fixtures/chanpy_stage1/input/stage1_small.csv",
        help="CSV with time,open,high,low,close,volume columns.",
    )
    parser.add_argument(
        "--out",
        default="fixtures/chanpy_stage1/gold/stage1_small_chanpy_gold.json",
        help="Output gold JSON path.",
    )
    parser.add_argument("--symbol", default="stage1_small")
    parser.add_argument("--market", default="LOCAL")
    parser.add_argument("--freq", default="DAILY")
    parser.add_argument("--adjust", default="QFQ")
    parser.add_argument("--mode", choices=["once", "step"], default="once")
    parser.add_argument("--config-json", default="", help="JSON object merged into analyze_bars config.")
    return parser.parse_args()


def read_bars(csv_path: Path) -> list[dict[str, Any]]:
    with csv_path.open("r", encoding="utf-8-sig", newline="") as f:
        reader = csv.DictReader(f)
        if reader.fieldnames is None:
            raise ValueError(f"CSV has no header: {csv_path}")
        rows: list[dict[str, Any]] = []
        for row in reader:
            if not row:
                continue
            rows.append(
                {
                    "dt": row.get("time") or row.get("dt") or row.get("date"),
                    "time": row.get("time") or row.get("dt") or row.get("date"),
                    "open": float(row["open"]),
                    "high": float(row["high"]),
                    "low": float(row["low"]),
                    "close": float(row["close"]),
                    "vol": float(row.get("volume") or row.get("vol") or 0),
                    "volume": float(row.get("volume") or row.get("vol") or 0),
                }
            )
        return rows


def import_hichan_engine(hichan_repo: Path):
    backend = hichan_repo / "backend"
    if not backend.exists():
        raise FileNotFoundError(f"hichan backend not found: {backend}")
    python_chanpy = hichan_repo / "python" / "chan.py"
    if python_chanpy.exists():
        os.environ.setdefault("CHANPY_PATH", str(python_chanpy))
    sys.path.insert(0, str(backend))
    sys.path.insert(0, str(hichan_repo))
    from app.chanpy_engine import analyze_bars  # type: ignore

    return analyze_bars


def compact_merged_bar(row: dict[str, Any]) -> dict[str, Any]:
    keys = [
        "index",
        "start_raw_index",
        "end_raw_index",
        "high_raw_index",
        "low_raw_index",
        "open",
        "high",
        "low",
        "close",
    ]
    return {key: row.get(key) for key in keys}


def compact_fx(row: dict[str, Any]) -> dict[str, Any]:
    return {
        "index": row.get("index"),
        "raw_index": row.get("raw_index"),
        "type": row.get("type"),
        "price": row.get("price"),
    }


def compact_bi(row: dict[str, Any]) -> dict[str, Any]:
    keys = [
        "index",
        "start_raw_index",
        "end_raw_index",
        "start_price",
        "end_price",
        "direction",
        "is_sure",
    ]
    return {key: row.get(key) for key in keys}


def compact_seg(row: dict[str, Any]) -> dict[str, Any]:
    keys = [
        "index",
        "start_bi_index",
        "end_bi_index",
        "start_raw_index",
        "end_raw_index",
        "start_price",
        "end_price",
        "direction",
        "is_sure",
    ]
    return {key: row.get(key) for key in keys}


def compact_zs(row: dict[str, Any]) -> dict[str, Any]:
    keys = [
        "index",
        "start_bi_index",
        "end_bi_index",
        "start_raw_index",
        "end_raw_index",
        "zg",
        "zd",
        "gg",
        "dd",
        "confirmed",
    ]
    return {key: row.get(key) for key in keys}


def compact_bsp(row: dict[str, Any]) -> dict[str, Any]:
    keys = [
        "index",
        "raw_index",
        "time",
        "price",
        "type",
        "level",
        "bi_index",
        "seg_index",
        "zs_index",
        "confirmed",
    ]
    return {key: row.get(key) for key in keys}


def normalize_gold(result: dict[str, Any]) -> dict[str, Any]:
    merged_bars = [compact_merged_bar(x) for x in result.get("merged_bars", [])]
    fx = [compact_fx(x) for x in result.get("fx", [])]
    bi = [compact_bi(x) for x in result.get("bi", [])]
    seg = [compact_seg(x) for x in result.get("seg", [])]
    segseg = [compact_seg(x) for x in result.get("segseg", [])]
    zs = [compact_zs(x) for x in result.get("zs", [])]
    seg_zs = [compact_zs(x) for x in result.get("seg_zs", [])]
    bsp = [compact_bsp(x) for x in result.get("bsp", [])]
    meta = result.get("meta") or {}
    return {
        "ok": bool(result.get("ok")),
        "meta": {
            "engine": meta.get("engine"),
            "symbol": meta.get("symbol"),
            "freq": meta.get("freq"),
            "adjust": meta.get("adjust"),
            "mode": meta.get("mode"),
            "gold_source": "hichan/chan.py",
            "generated_by": "tools/generate_chanpy_stage1_gold.py",
            "bars_count": len(result.get("bars", [])),
            "merged_bars_count": len(merged_bars),
            "fx_count": len(fx),
            "bi_count": len(bi),
            "segment_count": len(seg),
            "segseg_count": len(segseg),
            "zs_count": len(zs),
            "seg_zs_count": len(seg_zs),
            "bsp_count": len(bsp),
        },
        "merged_bars": merged_bars,
        "fx": fx,
        "bi": bi,
        "seg": seg,
        "segseg": segseg,
        "zs": zs,
        "seg_zs": seg_zs,
        "bsp": bsp,
    }


def main() -> int:
    args = parse_args()
    if not args.hichan_repo:
        raise SystemExit("Missing --hichan-repo or HICHAN_REPO")

    hichan_repo = Path(args.hichan_repo).resolve()
    input_path = Path(args.input)
    out_path = Path(args.out)
    bars = read_bars(input_path)
    analyze_bars = import_hichan_engine(hichan_repo)

    runtime_config: dict[str, Any] = {}
    if args.config_json:
        loaded_config = json.loads(args.config_json)
        if not isinstance(loaded_config, dict):
            raise SystemExit("--config-json must be a JSON object")
        runtime_config.update(loaded_config)

    result = analyze_bars(
        bars=bars,
        symbol=args.symbol,
        market=args.market,
        freq=args.freq,
        adjust=args.adjust,
        mode=args.mode,
        config=runtime_config,
    )
    gold = normalize_gold(result)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(gold, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"wrote {out_path}")
    print(
        "counts:",
        f"bars={gold['meta']['bars_count']}",
        f"merged_bars={gold['meta']['merged_bars_count']}",
        f"fx={gold['meta']['fx_count']}",
        f"bi={gold['meta']['bi_count']}",
        f"seg={gold['meta']['segment_count']}",
        f"segseg={gold['meta']['segseg_count']}",
        f"zs={gold['meta']['zs_count']}",
        f"seg_zs={gold['meta']['seg_zs_count']}",
        f"bsp={gold['meta']['bsp_count']}",
    )
    if not gold["ok"]:
        print(f"warning: chan.py returned fallback/error: {result.get('error')}")
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
