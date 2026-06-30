# chan.py BSP T3B fixture

## Purpose

Lock a concrete chan.py `3b` buy/sell point fixture for Rust parity.

This fixture was derived from `stage1_segseg_probe_candidate.csv` by rewriting the post-B1 next segment so that the pullback BI does not return into the previous comparison ZS.

## Files

- Input: `fixtures/chanpy_stage1/input/stage1_t3b_probe_candidate.csv`
- chan.py gold: `fixtures/chanpy_stage1/gold/stage1_t3b_probe_candidate_chanpy_gold.json`

## chan.py config

```json
{"bs_type":["1","3b"],"bsp3_follow_1":true,"bsp3_peak":false,"strict_bsp3":false,"bsp3a_max_zs_cnt":5}
```

## Expected T3B output

- Type: `B3b`
- Level: `bi`
- `bi_index`: 27
- `raw_index`: 142
- Time: `2027/07/02`
- Price: `16.05`

## Source-path notes

The relevant chan.py path is `CBSPointList.cal_seg_bs3point -> treat_bsp3_before -> add_bs(T3B)`.

Observed debug conditions for the parent B1 path:

- Parent segment: `seg=5`
- `bsp1_bi=25`
- Comparison ZS: `BI19~BI25`, `high=16.0`
- Candidate range starts at `BI27`
- `BI27` ends at `16.05`, so it no longer returns into the comparison ZS

## Known scope

This fixture locks structural T3B behavior. It does not attempt to cover every chan.py T3A/T3B edge case.
