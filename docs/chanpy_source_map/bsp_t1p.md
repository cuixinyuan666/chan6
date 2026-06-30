# chan.py BSP T1P / 1p source map

Purpose: lock local chan.py rules for T1P / 1p before Rust implementation.

Current Rust status:
- B1/S1 implemented.
- B2/S2 implemented.
- B2s/S2s implemented.
- BSP config supports enabled, types, follow options, and rate thresholds.
- T1P / 1p structural fallback is implemented for committed chan.py gold fixtures.

Source files:
- python/chan.py/BuySellPoint/BSPointList.py
- python/chan.py/BuySellPoint/BSPointConfig.py
- python/chan.py/Common/CEnum.py

Enum mapping:
- BSP_TYPE.T1  = 1
- BSP_TYPE.T1P = 1p
- BSP_TYPE.T2  = 2
- BSP_TYPE.T2S = 2s
- BSP_TYPE.T3A = 3a
- BSP_TYPE.T3B = 3b

Rust output mapping: buy T1P -> B1p, sell T1P -> S1p.

Entry path: CBSPointList.cal -> cal_seg_bs1point -> cal_single_bs1point.

T1P branch: cal_single_bs1point first tries normal T1. If the normal last-ZS T1 path is not satisfied, it falls back to treat_pz_bsp1.

T1P structural rule:
- last_bi = seg.end_bi.
- pre_bi = bi_list[last_bi.idx - 2].
- last_bi and pre_bi must belong to the same segment.
- last_bi direction must equal segment direction.
- Down-side T1P requires last_bi to make a new low versus pre_bi.
- Up-side T1P requires last_bi to make a new high versus pre_bi.

T1P divergence rule:
- in_metric = pre_bi.cal_macd_metric(macd_algo, is_reverse=False).
- out_metric = last_bi.cal_macd_metric(macd_algo, is_reverse=True).
- is_diver = out_metric <= divergence_rate * in_metric.

Important add_bs behavior: T1 and T1P are added to bsp1_list even when not final target output.

Current parity note: structural T1P fallback is implemented against committed chan.py gold fixtures. Exact MACD metric parity remains documented as a known gap.

Known gap: exact BI MACD metric parity is not available in Rust yet.
