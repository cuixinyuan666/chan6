# T1P / 1p fixture note

Fixture:
- input: fixtures/chanpy_stage1/input/stage1_t1p_probe_candidate.csv
- gold: fixtures/chanpy_stage1/gold/stage1_t1p_probe_candidate_chanpy_gold.json

Purpose: lock chan.py T1P / 1p output for Rust parity.

This fixture intentionally uses advanced BSP config so T1P is exported as a target BSP instead of only acting as an internal bsp1_list anchor.

Gold generation config:
- min_zs_cnt-buy = 0
- min_zs_cnt-sell = 0
- min_zs_cnt-segbuy = 0
- min_zs_cnt-segsell = 0
- bs_type-buy = 1p
- bs_type-sell = 1p
- bs_type-segbuy = 1p
- bs_type-segsell = 1p
- divergence_rate-buy = 1e999
- divergence_rate-sell = 1e999
- divergence_rate-segbuy = 1e999
- divergence_rate-segsell = 1e999

Expected BSP:
- one bi-level S1p
- raw_index = 13
- price = 15.0
- bi_index = 2
- no ZS is required

Covered chan.py path: CBSPointList.cal_single_bs1point -> treat_pz_bsp1 -> add_bs(T1P).
