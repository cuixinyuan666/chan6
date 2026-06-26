const chartQueryAtSampleJson = r'''
{
  "meta": {
    "schema_version": 1,
    "query": "query-chart-at",
    "symbol": "002003",
    "kline_scope": "target_centered_window",
    "day": 20260511,
    "minute": 1121,
    "before": 3,
    "after": 3,
    "window_offset": 1126616,
    "window_limit": 7,
    "kline_count": 7,
    "target_bar_id": 1126619,
    "target_index": 3,
    "chip_scope": "full_history_to_target_bar",
    "chip_bar_id": 1126619,
    "chip_truncated": true,
    "chip_top": 20
  },
  "symbol": "002003",
  "day": 20260511,
  "minute": 1121,
  "before": 3,
  "after": 3,
  "window_offset": 1126616,
  "window_limit": 7,
  "kline_count": 7,
  "target_bar_id": 1126619,
  "target_index": 3,
  "chip_bar_id": 1126619,
  "chip_scope": "full_history_to_target_bar",
  "chip_truncated": true,
  "chip_top": 20,
  "target": {
    "symbol": "002003",
    "bar_id": 1126619,
    "trading_day": 20260511,
    "minute": 1121,
    "open": 9.82,
    "high": 9.82,
    "low": 9.81,
    "close": 9.81,
    "volume": 9.0,
    "amount": 88.34
  },
  "kline": [
    {
      "symbol": "002003",
      "bar_id": 1126616,
      "trading_day": 20260511,
      "minute": 1118,
      "open": 9.78,
      "high": 9.80,
      "low": 9.77,
      "close": 9.79,
      "volume": 1200.0,
      "amount": 11748.0
    },
    {
      "symbol": "002003",
      "bar_id": 1126617,
      "trading_day": 20260511,
      "minute": 1119,
      "open": 9.79,
      "high": 9.83,
      "low": 9.78,
      "close": 9.82,
      "volume": 1800.0,
      "amount": 17676.0
    },
    {
      "symbol": "002003",
      "bar_id": 1126618,
      "trading_day": 20260511,
      "minute": 1120,
      "open": 9.82,
      "high": 9.84,
      "low": 9.80,
      "close": 9.82,
      "volume": 950.0,
      "amount": 9329.0
    },
    {
      "symbol": "002003",
      "bar_id": 1126619,
      "trading_day": 20260511,
      "minute": 1121,
      "open": 9.82,
      "high": 9.82,
      "low": 9.81,
      "close": 9.81,
      "volume": 9.0,
      "amount": 88.34
    },
    {
      "symbol": "002003",
      "bar_id": 1126620,
      "trading_day": 20260511,
      "minute": 1122,
      "open": 9.81,
      "high": 9.83,
      "low": 9.80,
      "close": 9.82,
      "volume": 700.0,
      "amount": 6874.0
    },
    {
      "symbol": "002003",
      "bar_id": 1126621,
      "trading_day": 20260511,
      "minute": 1123,
      "open": 9.82,
      "high": 9.85,
      "low": 9.82,
      "close": 9.84,
      "volume": 1600.0,
      "amount": 15744.0
    },
    {
      "symbol": "002003",
      "bar_id": 1126622,
      "trading_day": 20260511,
      "minute": 1124,
      "open": 9.84,
      "high": 9.86,
      "low": 9.83,
      "close": 9.85,
      "volume": 1100.0,
      "amount": 10835.0
    }
  ],
  "chip": [
    {
      "price_tick": 9670,
      "price": 9.67,
      "volume": 4200.0,
      "amount": 40614.0,
      "trade_count": 103
    },
    {
      "price_tick": 9730,
      "price": 9.73,
      "volume": 6800.0,
      "amount": 66164.0,
      "trade_count": 144
    },
    {
      "price_tick": 9810,
      "price": 9.81,
      "volume": 9200.0,
      "amount": 90252.0,
      "trade_count": 201
    },
    {
      "price_tick": 9860,
      "price": 9.86,
      "volume": 5100.0,
      "amount": 50286.0,
      "trade_count": 98
    }
  ]
}
''';
