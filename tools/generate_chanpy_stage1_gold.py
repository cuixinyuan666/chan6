#!/usr/bin/env python3
"""Generate chan.py gold output for Chan6 stage-1 Rust alignment.

This script does not implement Chan logic. It calls the hichan Python backend
(`backend/app/chanpy_engine.py::analyze_bars`) and writes the resulting
merged_bars/fx/bi structures as a gold JSON fixture.

Usage example:

    python tools/generate_chanpy_stage1_gold.py \
      --hichan-repo ../chan_replay_app \
      --input fixtures/chanpy_stage1/input/stage1_small.csv \
      --out fixtures/chanpy_stage1/gold/stage1_small_chanpy_gold.json
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


def normalize_gold(result: dict[str, Any], *, input_path: Path, hichan_repo: Path) -> dict[str, Any]:
    meta = dict(result.get("meta") or {})
    meta.update(
        {
            "gold_source": "hichan/chan.py",
            "hichan_repo": str(hichan_repo.resolve()),
            "input_csv": str(input_path),
            "generated_by": "tools/generate_chanpy_stage1_gold.py",
        }
    )
    return {
        "ok": bool(result.get("ok")),
        "meta": meta,
        "bars": result.get("bars", []),
        "merged_bars": result.get("merged_bars", []),
        "fx": result.get("fx", []),
        "bi": result.get("bi", []),
        "seg": result.get("seg", []),
        "zs": result.get("zs", []),
        "bsp": result.get("bsp", []),
        "frames": result.get("frames", []),
        "error": result.get("error"),
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
    result = analyze_bars(
        bars=bars,
        symbol=args.symbol,
        market=args.market,
        freq=args.freq,
        adjust=args.adjust,
        mode=args.mode,
        config={},
    )
    gold = normalize_gold(result, input_path=input_path, hichan_repo=hichan_repo)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(gold, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"wrote {out_path}")
    print(
        "counts:",
        f"bars={len(gold['bars'])}",
        f"merged_bars={len(gold['merged_bars'])}",
        f"fx={len(gold['fx'])}",
        f"bi={len(gold['bi'])}",
    )
    if not gold["ok"]:
        print(f"warning: chan.py returned fallback/error: {gold.get('error')}")
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
