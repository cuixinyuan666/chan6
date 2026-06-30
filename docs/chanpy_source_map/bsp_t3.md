# chan.py BSP T3A / T3B source map

Purpose: lock local chan.py rules for T3A / 3a and T3B / 3b before Rust implementation.

Current Rust status:
- B1/S1 implemented.
- B1p/S1p implemented for committed structural fallback fixtures.
- B2/S2 implemented.
- B2s/S2s implemented.
- T3A / 3a and T3B / 3b are not implemented yet.

Source files:
- python/chan.py/BuySellPoint/BSPointList.py
- python/chan.py/BuySellPoint/BSPointConfig.py
- python/chan.py/Common/CEnum.py

Enum mapping:
- BSP_TYPE.T3A = 3a. Comment in chan.py: 中枢在1类后面.
- BSP_TYPE.T3B = 3b. Comment in chan.py: 中枢在1类前面.

Rust output mapping:
- buy T3A -> B3a
- sell T3A -> S3a
- buy T3B -> B3b
- sell T3B -> S3b

Entry path:
- CBSPointList.cal -> cal_seg_bs3point.
- cal_seg_bs3point skips unless config target_types contains T3A or T3B.
- For each segment needing calculation:
  - If there is more than one segment, bsp1_bi = seg.end_bi, real_bsp1 = bsp1_dict.get(bsp1_bi.idx), next_seg = seg.next, next_seg_idx = seg.idx + 1.
  - Else next_seg = seg, bsp1_bi = None, real_bsp1 = None, bsp1_bi_idx = -1.
- If bsp3_follow_1 is true and bsp1_bi is missing or bsp1_bi is not in bsp_store_flat_dict, return.
- Then call treat_bsp3_after if next_seg exists.
- Then call treat_bsp3_before.

T3A / 3a path: treat_bsp3_after.
- Uses first_zs = next_seg.get_first_multi_bi_zs(); return if missing.
- If strict_bsp3 is true, require first_zs.get_bi_in().idx == bsp1_bi_idx + 1.
- Reads bsp3a_max_zs_cnt from config for next_seg direction.
- Iterate next_seg.get_multi_bi_zs_lst(), capped by bsp3a_max_zs_cnt.
- For each ZS:
  - break if zs.bi_out is missing or zs.bi_out.idx + 1 >= len(bi_list).
  - bsp3_bi = bi_list[zs.bi_out.idx + 1].
  - If bsp3_bi has no parent segment and next_seg is not the last segment, break.
  - If bsp3_bi parent segment differs from next_seg and parent segment has at least 3 BI, break.
  - If bsp3_bi direction equals next_seg direction, break.
  - If bsp3_bi.seg_idx != next_seg_idx and next_seg_idx < len(seg_list) - 2, break.
  - If bsp3_bi returns back into ZS range, continue.
  - If bsp3_peak is true and bsp3_bi does not break ZS peak, continue.
  - Add T3A at bsp3_bi.

T3B / 3b path: treat_bsp3_before.
- Uses cmp_zs = seg.get_final_multi_bi_zs(); return if missing.
- Return if bsp1_bi is missing.
- If strict_bsp3 is true, require cmp_zs.bi_out exists and cmp_zs.bi_out.idx == bsp1_bi.idx.
- end_bi_idx = cal_bsp3_bi_end_idx(next_seg).
- Iterate bi_list[bsp1_bi.idx + 2 :: 2].
- For each candidate bsp3_bi:
  - break if bsp3_bi.idx > end_bi_idx.
  - require bsp3_bi.seg_idx exists.
  - break if bsp3_bi.seg_idx != next_seg_idx and bsp3_bi.seg_idx < len(seg_list) - 1.
  - If bsp3_bi returns back into cmp_zs range, continue.
  - Add T3B at bsp3_bi and break.

Helper predicates:
- bsp3_back2zs(bsp3_bi, zs):
  - down candidate returns to ZS if bsp3_bi._low() < zs.high.
  - up candidate returns to ZS if bsp3_bi._high() > zs.low.
- bsp3_break_zspeak(bsp3_bi, zs):
  - down candidate breaks peak if bsp3_bi._high() >= zs.peak_high.
  - up candidate breaks peak if bsp3_bi._low() <= zs.peak_low.

Relevant config keys:
- bs_type: includes 3a and 3b by default.
- bsp3_follow_1: default true.
- bsp3_peak: default false.
- strict_bsp3: default false.
- bsp3a_max_zs_cnt: default 1.

Implementation plan:
- First generate chan.py gold fixture that exposes B3a/S3a or B3b/S3b.
- Prefer config-json bs_type override to isolate 3a or 3b.
- Then implement minimal Rust parity against committed fixture.
- Keep strict_bsp3, bsp3_peak, and exact feature metrics as documented later-stage gaps unless covered by fixture.

Known gaps before Rust implementation:
- No committed chan.py gold fixture currently exposes B3a/S3a/B3b/S3b.
- Rust currently recognizes B3a/S3a/B3b/S3b in output filtering, but does not construct them.
