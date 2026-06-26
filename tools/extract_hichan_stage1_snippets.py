#!/usr/bin/env python3
"""Extract small hichan chan.py source snippets for stage-1 Rust audit.

The script only reads local text files from a checked-out hichan repo. It does
not import or execute hichan code.
"""

from __future__ import annotations

import argparse
import os
import re
from pathlib import Path

TARGETS = {
    "Combiner/KLine_Combiner.py": [
        "class CKLine_Combiner",
        "def test_combine",
        "def try_add",
        "def update_fx",
        "def get_peak_klu",
    ],
    "KLine/KLine.py": [
        "class CKLine",
        "def set_fx",
        "def check_fx_valid",
        "def get_peak_klu",
    ],
    "KLine/KLine_List.py": [
        "class CKLine_List",
        "def add_single_klu",
        "def cal_seg_and_zs",
        "def try_add",
    ],
    "Bi/Bi.py": [
        "class CBi",
        "def get_begin_klu",
        "def get_end_klu",
        "def get_begin_val",
        "def get_end_val",
    ],
    "Bi/BiConfig.py": [
        "class CBiConfig",
    ],
    "Bi/BiList.py": [
        "class CBiList",
        "def try_add",
        "def can_make_bi",
        "def end_is_peak",
        "def update_bi",
        "def add_new_bi",
    ],
    "Common/CEnum.py": [
        "class FX_TYPE",
        "class BI_DIR",
    ],
    "Common/func_util.py": [
        "def revert_bi_dir",
    ],
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--hichan-repo", default=os.environ.get("HICHAN_REPO"))
    parser.add_argument("--out", default="docs/chan_py_stage1_source_snippets.md")
    parser.add_argument("--context", type=int, default=45)
    return parser.parse_args()


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return path.read_text(encoding="utf-8", errors="ignore")


def find_marker_line(lines: list[str], marker: str) -> int | None:
    if marker.startswith("def "):
        pattern = re.compile(rf"^\s*{re.escape(marker)}\s*\(")
    elif marker.startswith("class "):
        pattern = re.compile(rf"^\s*{re.escape(marker)}\b")
    else:
        pattern = re.compile(re.escape(marker))
    for idx, line in enumerate(lines):
        if pattern.search(line):
            return idx
    return None


def snippet(lines: list[str], start: int, context: int) -> tuple[int, int, str]:
    lo = max(0, start - 3)
    hi = min(len(lines), start + context)
    body = "".join(f"{line_no + 1:04d}: {lines[line_no]}" for line_no in range(lo, hi))
    return lo + 1, hi, body.rstrip()


def main() -> int:
    args = parse_args()
    if not args.hichan_repo:
        raise SystemExit("Missing --hichan-repo or HICHAN_REPO")
    hichan_repo = Path(args.hichan_repo).resolve()
    root = hichan_repo / "python" / "chan.py"
    if not root.exists():
        raise SystemExit(f"python/chan.py directory not found: {root}")

    sections: list[str] = [
        "# hichan chan.py stage-1 source snippets",
        "",
        "Generated from local hichan checkout. This file may be committed only after removing local absolute paths if any are added manually.",
        "",
    ]

    for rel_path, markers in TARGETS.items():
        path = root / rel_path
        sections.append(f"## {rel_path}")
        sections.append("")
        if not path.exists():
            sections.append("```text")
            sections.append("missing")
            sections.append("```")
            sections.append("")
            continue
        lines = read_text(path).splitlines(keepends=True)
        for marker in markers:
            line_idx = find_marker_line(lines, marker)
            sections.append(f"### {marker}")
            sections.append("")
            if line_idx is None:
                sections.append("```text")
                sections.append("not found")
                sections.append("```")
                sections.append("")
                continue
            lo, hi, body = snippet(lines, line_idx, args.context)
            sections.append(f"Lines {lo}-{hi}")
            sections.append("")
            sections.append("```python")
            sections.append(body)
            sections.append("```")
            sections.append("")

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(sections), encoding="utf-8")
    print(f"wrote {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
