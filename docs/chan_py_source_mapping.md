# hichan chan.py source mapping

This document records the stable relative source-file map discovered from the local hichan checkout.

The local scan was produced by:

```bash
HICHAN_REPO="$HOME/Downloads/chan_replay_app_v0_1/chan_replay_app" \
python tools/scan_hichan_chanpy_sources.py \
  --out docs/chan_py_local_source_map.json
```

The generated `docs/chan_py_local_source_map.json` contains local absolute paths and should not be committed. This file keeps only stable relative paths and audit decisions.

## Scan summary

```text
candidate_source_files=45
root=python/chan.py
```

## Stage-1 authoritative source map

### Runtime entrance

| Purpose | hichan source |
|---|---|
| Top-level chan.py container | `Chan.py` |
| Runtime config | `ChanConfig.py` |
| Common enums | `Common/CEnum.py` |
| Common BI helpers | `Common/func_util.py` |

### Include / merged K-line

| Purpose | hichan source |
|---|---|
| Combined K-line item | `Combiner/Combine_Item.py` |
| Include merge engine | `Combiner/KLine_Combiner.py` |
| K-line object with FX state | `KLine/KLine.py` |
| K-line list pipeline | `KLine/KLine_List.py` |
| Raw K-line unit | `KLine/KLine_Unit.py` |

Rust files that must align:

```text
crates/chan6_core/src/chan/include.rs
crates/chan6_core/src/chan/engine.rs
```

Audit notes:

```text
1. Do not treat include merge as an isolated high/low comparison only.
2. KLine_Combiner.py is the source for test_combine / try_add behavior.
3. KLine/KLine.py and KLine_List.py determine how combined bars expose FX and downstream structures.
4. Raw index exported by hichan maps to Chan6 bar_id.
```

### FX / fractal

| Purpose | hichan source |
|---|---|
| FX enum | `Common/CEnum.py` |
| Combined K-line FX state | `Combiner/KLine_Combiner.py` |
| K-line FX exposure and checks | `KLine/KLine.py` |
| K-line list structure update | `KLine/KLine_List.py` |

Rust files that must align:

```text
crates/chan6_core/src/chan/fx.rs
crates/chan6_core/src/chan/engine.rs
```

Audit notes:

```text
1. Rust FX output must match hichan export_fx output.
2. hichan raw_index maps to Rust bar_id.
3. hichan FX index maps to Rust merged_index for exported fixture comparison.
4. Price anchor must follow hichan top/bottom anchor, not a guessed rule.
```

### BI / stroke

| Purpose | hichan source |
|---|---|
| BI object | `Bi/Bi.py` |
| BI config | `Bi/BiConfig.py` |
| BI list construction | `Bi/BiList.py` |
| BI enum | `Common/CEnum.py` |
| BI helper functions | `Common/func_util.py` |
| K-line list orchestration | `KLine/KLine_List.py` |

Rust files that must align:

```text
crates/chan6_core/src/chan/bi.rs
crates/chan6_core/src/chan/engine.rs
```

Audit notes:

```text
1. Rust BI construction must eventually map to CBiList.try_add behavior.
2. BI config values bi_algo / bi_strict / bi_fx_check must map into ChanConfig before production use.
3. start_raw_index / end_raw_index map to start_bar_id / end_bar_id.
4. is_sure maps to confirmed.
5. Direction must follow hichan BI_DIR semantics.
```

## Later source map

### Segment

| Purpose | hichan source |
|---|---|
| Segment object | `Seg/Seg.py` |
| Segment config | `Seg/SegConfig.py` |
| Chan segment list | `Seg/SegListChan.py` |
| Common segment list | `Seg/SegListComm.py` |
| Default segment list | `Seg/SegListDef.py` |
| DYH segment list | `Seg/SegListDYH.py` |
| Eigen structures | `Seg/Eigen.py`, `Seg/EigenFX.py` |

### Zhongshu

| Purpose | hichan source |
|---|---|
| Zhongshu object | `ZS/ZS.py` |
| Zhongshu config | `ZS/ZSConfig.py` |
| Zhongshu list | `ZS/ZSList.py` |

### BSP / plotting evidence

| Purpose | hichan source |
|---|---|
| BSP point | `BuySellPoint/BS_Point.py` |
| BSP list | `BuySellPoint/BSPointList.py` |
| Plot metadata export reference | `Plot/PlotMeta.py` |
| Plot driver reference | `Plot/PlotDriver.py` |

## Immediate engineering rule

Before changing Rust include / fx / bi again:

```text
1. Use hichan gold fixtures as executable tests.
2. Use this source map to locate the exact hichan source file.
3. Record which hichan class/method the Rust function corresponds to.
4. Do not expand CLI or frontend until stage-1 include/fx/bi has at least one committed gold fixture per key behavior.
```
