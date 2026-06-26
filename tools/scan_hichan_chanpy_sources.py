#!/usr/bin/env python3
"""Scan local hichan python/chan.py source files for Rust alignment.

This script only reads text files. It does not import or execute hichan code.
"""

from __future__ import annotations

import argparse
import json
import os
import re
from pathlib import Path

KEYWORDS = [
    "class CChan",
    "class CChanConfig",
    "class CKLine",
    "class CKLine_List",
    "class CKLine_Combiner",
    "class CBi",
    "class CBiList",
    "class CSeg",
    "class CSegList",
    "class CZS",
    "class CZSList",
    "FX_TYPE",
    "BI_DIR",
    "bi_algo",
    "bi_strict",
    "bi_fx_check",
    "get_peak_klu",
    "get_begin_klu",
    "get_end_klu",
    "test_combine",
    "try_add",
]

PATH_HINTS = ["Chan", "ChanConfig", "KLine", "Combiner", "Bi", "Seg", "ZS", "Common"]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--hichan-repo", default=os.environ.get("HICHAN_REPO"))
    parser.add_argument("--out", default="docs/chan_py_local_source_map.json")
    return parser.parse_args()


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return path.read_text(encoding="utf-8", errors="ignore")


def scan_file(path: Path, root: Path) -> dict | None:
    text = read_text(path)
    rel = path.relative_to(root).as_posix()
    matched_keywords = [kw for kw in KEYWORDS if kw in text]
    hinted_path = any(hint.lower() in rel.lower() for hint in PATH_HINTS)
    if not matched_keywords and not hinted_path:
        return None
    return {
        "path": rel,
        "matched_keywords": matched_keywords,
        "classes": re.findall(r"^class\s+([A-Za-z_][A-Za-z0-9_]*)", text, flags=re.MULTILINE),
        "functions": re.findall(r"^def\s+([A-Za-z_][A-Za-z0-9_]*)", text, flags=re.MULTILINE),
    }


def main() -> int:
    args = parse_args()
    if not args.hichan_repo:
        raise SystemExit("Missing --hichan-repo or HICHAN_REPO")
    hichan_repo = Path(args.hichan_repo).resolve()
    root = hichan_repo / "python" / "chan.py"
    if not root.exists():
        raise SystemExit(f"python/chan.py directory not found: {root}")
    rows = []
    for path in sorted(root.rglob("*.py")):
        row = scan_file(path, root)
        if row is not None:
            rows.append(row)
    output = {
        "hichan_repo": str(hichan_repo),
        "chanpy_root": str(root),
        "candidate_source_files": rows,
    }
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(output, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"wrote {out_path}")
    print(f"candidate_source_files={len(rows)}")
    for row in rows[:80]:
        print(row["path"], "keywords=", ",".join(row["matched_keywords"][:8]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
