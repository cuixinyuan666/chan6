# chan6

`chan6` 第一阶段只使用离线逐笔数据，不联网，不使用 AKShare / 东财 / Tushare。

当前最小闭环：

```text
离线逐笔 CSV
  ├─ Rust 合成 1min K线
  └─ Rust 按逐笔成交价累加筹码分布
        ├─ 每根1min K保存筹码增量
        └─ 每 N 根K保存一次完整筹码快照
```

## 目标

- Windows 和 Android 共用同一套 Rust 核心算法。
- 1min K线可以由逐笔合成。
- 筹码分布只能由逐笔累加，不能用 1min K线反推。
- 十字辅助线定位到某根K时，查询该K收盘后的累计筹码状态。

## 支持的 CSV 表头

第一版会自动识别常见中英文表头：

| 字段 | 支持表头 |
|---|---|
| 股票代码 | `symbol`, `code`, `股票代码`, `证券代码`, `代码` |
| 成交时间 | `datetime`, `time`, `成交时间`, `时间`, `日期时间` |
| 成交价 | `price`, `成交价`, `成交价格`, `最新价` |
| 成交量 | `volume`, `vol`, `qty`, `成交量`, `数量` |
| 成交额 | `amount`, `成交额`, `成交金额`，可选 |

如果 CSV 没有股票代码列，可以用 `--symbol` 指定。

## 编译

```bash
cargo build
```

## 导入逐笔 CSV

```bash
cargo run -p chan6_cli -- import-tick \
  --csv ./data/ticks/000001.csv \
  --db ./data/cache/chan6_kline_chip.db \
  --symbol 000001 \
  --replace
```

默认：

- 价格精度 `--price-scale 1000`，即 `10.235 -> 10235`。
- 每 `60` 根 1min K 保存一次完整筹码快照。

## 查询 1min K线

```bash
cargo run -p chan6_cli -- query-kline \
  --db ./data/cache/chan6_kline_chip.db \
  --symbol 000001 \
  --limit 20
```

## 查询某根K对应的筹码分布

```bash
cargo run -p chan6_cli -- query-chip \
  --db ./data/cache/chan6_kline_chip.db \
  --symbol 000001 \
  --bar-id 120
```

返回的是从逐笔数据开始到该 `bar_id` 结束时的累计筹码状态。

## SQLite 表

- `kline_1m`：逐笔合成的 1min K线。
- `chip_delta_1m`：每根K内部逐笔成交产生的筹码增量。
- `chip_snapshot`：定期保存的完整筹码快照，用于快速查询十字线定位时的筹码状态。
