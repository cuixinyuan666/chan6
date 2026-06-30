# chan.py BSP T3A fixture

## Purpose

Lock a concrete chan.py `3a` buy/sell point fixture for Rust parity.

This fixture was derived from `stage1_segseg_probe_candidate.csv` by extending the tail so the post-ZS pullback BI does not return into the first ZS of the next segment.

## Files

- Input: `fixtures/chanpy_stage1/input/stage1_t3a_probe_candidate.csv`
- chan.py gold: `fixtures/chanpy_stage1/gold/stage1_t3a_probe_candidate_chanpy_gold.json`

## chan.py config

```json
{"bs_type":["1","3a"],"bsp3_follow_1":true,"bsp3_peak":false,"strict_bsp3":false,"bsp3a_max_zs_cnt":5}
```

## Expected T3A output

- Type: `B3a`
- Level: `bi`
- `bi_index`: 31
- `raw_index`: 170
- Time: `2027/07/30`
- Price: `25.5`

## Source-path notes

The relevant chan.py path is `CBSPointList.cal_seg_bs3point -> treat_bsp3_after -> add_bs(T3A)`.

Observed trigger path:

- Parent B1 BI: `BI25`
- Next segment: `seg=6`, `BI26~BI30`
- First next-segment ZS: `BI27~BI29`, `high=22.0`, `low=5.5`
- Candidate: `BI31`, down, `raw166->170`, `30.8->25.5`
- Candidate low `25.5` is above ZS high `22.0`, so it does not return into the ZS

## Known scope

This fixture locks structural T3A behavior for the committed chan.py gold case. It does not cover every peak-filter or strict-mode edge case.
