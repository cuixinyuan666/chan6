# query_chan_basic JSON contract

Schema version: 1.

`query_chan_basic` returns a JSON payload for Rust Chan stage-1 analysis and frontend drawing.

## CLI

Command:

cargo run -p chan6_cli --bin query_chan_basic -- --db <sqlite-db> --symbol <symbol> --offset <offset> --limit <limit> --level <level>

Arguments:

| Argument | Meaning |
| --- | --- |
| --db | SQLite database path. |
| --symbol | Query symbol. |
| --offset | Raw kline offset. Default: 0. |
| --limit | Raw kline limit. Default: 300. |
| --level | Display/calculation level label. Default: 1m. |

## Top-level keys

| Key | Meaning |
| --- | --- |
| meta | CLI metadata and render hints. |
| kline | Raw queried KLine1m rows. |
| chan_basic | Canonical full Chan stage-1 snapshot. |
| merged_boxes | Render shortcut for merged bars. |
| fx_lines | Render shortcut for FX points. |
| bi_lines | Render shortcut for BI lines. |
| segment_lines | Render shortcut for segment lines. |

`chan_basic` is the canonical algorithm output. The other render arrays are convenience layers derived from `chan_basic`.

## chan_basic fields

| Field | Meaning |
| --- | --- |
| meta | Core snapshot metadata. |
| bars | Raw bars converted to Chan bars. |
| merged_bars | Include-merged bars. |
| fx | Top/bottom fractals. |
| bi | BI strokes. |
| segments | Line segments and higher segment layers. |
| zs | BI-level centers. |
| seg_zs | Segment-level centers. |
| bsp | Buy/sell points. |

## Index semantics

| Field | Meaning |
| --- | --- |
| index | Sequence index inside the current array unless documented otherwise. |
| bar_id | Raw kline anchor used for drawing and cross-reference. |
| start_bar_id / end_bar_id | Raw kline anchors for a range or line endpoint. |
| merged_index | Index into chan_basic.merged_bars. |
| start_fx_index / end_fx_index | BI endpoint FX indices into chan_basic.fx. |
| parent_segment_index | Parent segment index when present. |
| start_parent_index / end_parent_index | Segment parent range. For n=1, parent object is BI. |
| start_bi_index / end_bi_index | ZS BI range, or render alias for segment BI range. |
| start_segment_index / end_segment_index | Segment-level ZS range. |
| bi_index | BSP parent BI index when applicable. |
| segment_index | BSP parent segment index when applicable. |

BSP note: BSP rows are sorted for display by raw bar, but BSP `index` follows chan.py-compatible generation order.

## Drawing rules

| Layer | Drawing anchor |
| --- | --- |
| merged_boxes | start_bar_id/end_bar_id/high/low |
| fx_lines | bar_id/price |
| bi_lines | start_bar_id/start_price -> end_bar_id/end_price |
| segment_lines | start_bar_id/start_price -> end_bar_id/end_price |
| zs | start_bar_id/end_bar_id/zg/zd/gg/dd |
| seg_zs | start_bar_id/end_bar_id/zg/zd/gg/dd |
| bsp | bar_id/price/type |

For merged bars, `high` and `low` are the visual envelope. `calc_high` and `calc_low` are internal algorithm fields.

## Compatibility rule

Frontend consumers should:

1. Use `chan_basic` as the source of truth.
2. Use merged_boxes, fx_lines, bi_lines, and segment_lines only as drawing shortcuts.
3. Treat new fields as additive.
4. Do not rename or remove fields without bumping schema_version.
