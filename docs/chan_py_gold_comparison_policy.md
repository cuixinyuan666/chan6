# chan.py gold comparison policy

Chan6 Rust Chan implementation must be comparison-first.

Every Rust-generated Chan element must be validated against `chan_replay_app` branch `hichan` / `python/chan.py` before it is treated as stable.

## Hard rule

A Rust Chan element is not considered implemented unless all of the following exist:

```text
1. hichan source mapping
2. local hichan gold generation path
3. committed input fixture
4. committed chan.py gold fixture
5. Rust executable test comparing Rust output to chan.py gold
6. documented known gaps if exact parity is not complete
```

No frontend, CLI, or higher-order element should rely on a Rust Chan output that has no chan.py gold comparison.

## Fixture naming

Use stable fixture names:

```text
fixtures/chanpy_stage1/input/<case>.csv
fixtures/chanpy_stage1/gold/<case>_chanpy_gold.json
```

For later stages:

```text
fixtures/chanpy_stage2/input/<case>.csv
fixtures/chanpy_stage2/gold/<case>_chanpy_gold.json
```

## Current stage-1 comparison coverage

| Rust element / rule | Source area in hichan | Gold status | Rust test status |
|---|---|---:|---:|
| raw K -> ChanBar anchor model | model/KLine export | covered by model tests | covered |
| include merge: basic containment | Combiner/KLine_Combiner.py | covered | covered |
| include merge: hichan envelope vs calc high/low split | Combiner/KLine_Combiner.py + exporter | covered by `stage1_end_is_peak_fail_candidate` | covered |
| FX strict top/bottom | CKLine_Combiner.update_fx | covered | covered |
| FX anchor uses internal calc peak KLU | CKLine.get_peak_klu / exporter | covered by `stage1_end_is_peak_fail_candidate` | covered |
| BI strict span success | CBiList.satisfy_bi_span | covered by `stage1_bi_candidate` | covered |
| BI strict span failure | CBiList.satisfy_bi_span | covered by `stage1_small` | covered |
| BI end-is-peak / gate failure class | CBiList.can_make_bi / end_is_peak | covered by `stage1_end_is_peak_fail_candidate` | covered |
| BI check_fx_valid gate | CKLine.check_fx_valid | missing | missing |
| BI endpoint update/replacement | CBiList.update_bi_sure / update_bi | missing | missing |
| bi_algo='fx' | Bi/BiConfig.py / CBiList.satisfy_bi_span | missing | missing |
| non-strict BI | Bi/BiConfig.py / CBiList.satisfy_bi_span | missing | missing |
| step / virtual BI | KLine_List.add_single_klu / CBiList.update_bi | missing | missing |

## Later-stage coverage requirements

Before implementing a Rust module as usable output, create chan.py comparison fixtures for:

```text
Segment / 线段:
  - basic line segment formation
  - line segment extension/replacement
  - line segment break
  - uncertain / unconfirmed segment if chan.py exports it

Segseg / 2段:
  - basic segseg formation
  - default max derivable N behavior

ZS / 中枢:
  - BI-level ZS
  - Segment-level ZS
  - combine / non-combine behavior

BSP / 买卖点:
  - first/second/third buy/sell point classes
  - level linkage
  - anti-future step validation

Rhythm 1382 and higher overlays:
  - backend-generated anchors only
  - bar_id + price anchors only
  - no screen-coordinate business state
```

## Frontend rule

Flutter only renders Rust output. It must not generate or mutate Chan business elements.

Allowed in Flutter:

```text
1. map `bar_id + price` to viewport coordinates
2. draw merged boxes from Rust `merged_boxes`
3. draw FX lines from Rust `fx_lines`
4. draw BI/segment/ZS/BSP overlays from Rust output
5. user-created visual drawings separate from business overlays
```

Not allowed in Flutter:

```text
1. recalculating include / FX / BI / Segment / ZS / BSP
2. storing business anchors as screen coordinates
3. deriving business truth from canvas state
```

## Required workflow for each new Chan element

```text
1. Add or identify hichan source snippets.
2. Add a candidate input fixture.
3. Generate chan.py gold locally through `tools/generate_chanpy_stage1_gold.py` or a later-stage generator.
4. Commit the gold fixture.
5. Add Rust comparison test.
6. If the test fails, fix Rust until the gold test passes.
7. Only then expose the element to CLI / Flutter.
```
