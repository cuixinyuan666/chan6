import 'dart:convert';
import 'dart:math' as math;

import '../core/chart_models.dart';

class ChartQueryParser {
  const ChartQueryParser._();

  static ChartState parseJsonString(String raw) {
    final decoded = jsonDecode(raw);
    if (decoded is! Map<String, dynamic>) {
      throw const FormatException('chart query JSON root must be an object');
    }
    return parseMap(decoded);
  }

  static ChartState parseMap(Map<String, dynamic> data) {
    final metaMap = _asStringMap(data['meta']);
    final symbol = _asString(metaMap['symbol'] ?? data['symbol'], 'UNKNOWN');

    final kline = _asList(data['kline'])
        .map((item) => _parseKLine(_asStringMap(item), fallbackSymbol: symbol))
        .toList(growable: false);

    final chip = _asList(data['chip'])
        .map((item) => _parseChipLevel(_asStringMap(item)))
        .toList(growable: false);

    final mergedBoxes = _asList(data['merged_boxes'])
        .map((item) => _parseMergedBox(_asStringMap(item)))
        .toList(growable: false);

    final fxLines = _asList(data['fx_lines'])
        .map((item) => _parseFxLinePoint(_asStringMap(item)))
        .toList(growable: false);

    final biLines = _asList(data['bi_lines'])
        .map((item) => _parseBiLinePoint(_asStringMap(item)))
        .toList(growable: false);

    final segmentLines = _asList(data['segment_lines'])
        .map((item) => _parseSegmentLinePoint(_asStringMap(item)))
        .toList(growable: false);

    final viewport = _buildInitialViewport(kline, mergedBoxes);

    final meta = ChartMeta(
      schemaVersion: _asInt(metaMap['schema_version'], 1),
      query: _asString(metaMap['query'], _asString(data['query'], 'unknown')),
      symbol: symbol,
      klineScope: _asString(metaMap['kline_scope'], 'unknown'),
      chipScope: _asString(
        metaMap['chip_scope'] ?? data['chip_scope'],
        'unknown',
      ),
      chipBarId: _asNullableInt(metaMap['chip_bar_id'] ?? data['chip_bar_id']),
      chipTruncated: _asBool(
        metaMap['chip_truncated'] ?? data['chip_truncated'],
        false,
      ),
      chipTop: _asInt(metaMap['chip_top'] ?? data['chip_top'], 0),
      targetBarId: _asNullableInt(
        metaMap['target_bar_id'] ?? data['target_bar_id'],
      ),
      targetIndex: _asNullableInt(
        metaMap['target_index'] ?? data['target_index'],
      ),
      klineCount: _asInt(
        metaMap['kline_count'] ?? data['kline_count'],
        kline.length,
      ),
      mergedCount: _asInt(metaMap['merged_count'], mergedBoxes.length),
      fxCount: _asInt(metaMap['fx_count'], fxLines.length),
      biCount: _asInt(metaMap['bi_count'], biLines.length),
      segmentCount: _asInt(metaMap['segment_count'], segmentLines.length),
      offset: _asNullableInt(metaMap['offset'] ?? data['offset']),
      limit: _asNullableInt(metaMap['limit'] ?? data['limit']),
      day: _asNullableInt(metaMap['day'] ?? data['day']),
      minute: _asNullableInt(metaMap['minute'] ?? data['minute']),
      before: _asNullableInt(metaMap['before'] ?? data['before']),
      after: _asNullableInt(metaMap['after'] ?? data['after']),
      windowOffset: _asNullableInt(
        metaMap['window_offset'] ?? data['window_offset'],
      ),
      windowLimit: _asNullableInt(
        metaMap['window_limit'] ?? data['window_limit'],
      ),
    );

    return ChartState(
      symbol: symbol,
      kline: kline,
      chip: chip,
      mergedBoxes: mergedBoxes,
      fxLines: fxLines,
      biLines: biLines,
      segmentLines: segmentLines,
      meta: meta,
      viewport: viewport,
      crosshair: CrosshairState.hidden,
      drawings: const [],
    );
  }

  static KLinePoint _parseKLine(
    Map<String, dynamic> item, {
    required String fallbackSymbol,
  }) {
    return KLinePoint(
      symbol: _asString(item['symbol'], fallbackSymbol),
      barId: _asInt(item['bar_id'], 0),
      tradingDay: _asInt(item['trading_day'], 0),
      minute: _asInt(item['minute'], 0),
      open: _asDouble(item['open'], 0),
      high: _asDouble(item['high'], 0),
      low: _asDouble(item['low'], 0),
      close: _asDouble(item['close'], 0),
      volume: _asDouble(item['volume'], 0),
      amount: _asDouble(item['amount'], 0),
    );
  }

  static ChipLevel _parseChipLevel(Map<String, dynamic> item) {
    return ChipLevel(
      priceTick: _asInt(item['price_tick'], 0),
      price: _asDouble(item['price'], 0),
      volume: _asDouble(item['volume'], 0),
      amount: _asDouble(item['amount'], 0),
      tradeCount: _asInt(item['trade_count'], 0),
    );
  }

  static MergedBox _parseMergedBox(Map<String, dynamic> item) {
    return MergedBox(
      index: _asInt(item['index'], 0),
      startBarId: _asInt(item['start_bar_id'], 0),
      endBarId: _asInt(item['end_bar_id'], 0),
      high: _asDouble(item['high'], 0),
      low: _asDouble(item['low'], 0),
      isMerged: _asBool(item['is_merged'], false),
      highBarId: _asInt(item['high_bar_id'], 0),
      lowBarId: _asInt(item['low_bar_id'], 0),
      calcHigh: _asDouble(item['calc_high'], 0),
      calcLow: _asDouble(item['calc_low'], 0),
      calcHighBarId: _asInt(item['calc_high_bar_id'], 0),
      calcLowBarId: _asInt(item['calc_low_bar_id'], 0),
    );
  }

  static FxLinePoint _parseFxLinePoint(Map<String, dynamic> item) {
    return FxLinePoint(
      index: _asInt(item['index'], 0),
      kind: _asString(item['kind'], 'unknown'),
      mergedIndex: _asInt(item['merged_index'], 0),
      barId: _asInt(item['bar_id'], 0),
      price: _asDouble(item['price'], 0),
      confirmed: _asBool(item['confirmed'], true),
    );
  }

  static BiLinePoint _parseBiLinePoint(Map<String, dynamic> item) {
    return BiLinePoint(
      index: _asInt(item['index'], 0),
      direction: _asString(item['direction'], 'unknown'),
      startBarId: _asInt(item['start_bar_id'], 0),
      startPrice: _asDouble(item['start_price'], 0),
      endBarId: _asInt(item['end_bar_id'], 0),
      endPrice: _asDouble(item['end_price'], 0),
      confirmed: _asBool(item['confirmed'], true),
    );
  }

  static SegmentLinePoint _parseSegmentLinePoint(Map<String, dynamic> item) {
    return SegmentLinePoint(
      index: _asInt(item['index'], 0),
      n: _asInt(item['n'], 1),
      inputN: _asInt(item['input_n'], 1),
      direction: _asString(item['direction'], 'unknown'),
      startBiIndex: _asNullableInt(item['start_bi_index']),
      endBiIndex: _asNullableInt(item['end_bi_index']),
      startBarId: _asInt(item['start_bar_id'], 0),
      startPrice: _asDouble(item['start_price'], 0),
      endBarId: _asInt(item['end_bar_id'], 0),
      endPrice: _asDouble(item['end_price'], 0),
      confirmed: _asBool(item['confirmed'], true),
      reason: _asString(item['reason'], ''),
    );
  }

  static ChartViewport _buildInitialViewport(
    List<KLinePoint> kline,
    List<MergedBox> mergedBoxes,
  ) {
    if (kline.isEmpty) {
      return const ChartViewport(
        startIndex: 0,
        endIndex: 0,
        minPrice: 0,
        maxPrice: 1,
      );
    }

    var minPrice = kline.first.low;
    var maxPrice = kline.first.high;

    for (final bar in kline) {
      minPrice = math.min(minPrice, bar.low);
      maxPrice = math.max(maxPrice, bar.high);
    }

    for (final box in mergedBoxes) {
      minPrice = math.min(minPrice, box.low);
      maxPrice = math.max(maxPrice, box.high);
    }

    final range = math.max(0.000001, maxPrice - minPrice);
    final padding = math.max(0.01, range * 0.06);

    return ChartViewport(
      startIndex: 0,
      endIndex: kline.length - 1,
      minPrice: minPrice - padding,
      maxPrice: maxPrice + padding,
    );
  }

  static Map<String, dynamic> _asStringMap(Object? value) {
    if (value is Map<String, dynamic>) {
      return value;
    }
    if (value is Map) {
      return value.map((key, value) => MapEntry(key.toString(), value));
    }
    return const {};
  }

  static List<dynamic> _asList(Object? value) {
    if (value is List) {
      return value;
    }
    return const [];
  }

  static String _asString(Object? value, String fallback) {
    if (value == null) {
      return fallback;
    }
    return value.toString();
  }

  static int _asInt(Object? value, int fallback) {
    return _asNullableInt(value) ?? fallback;
  }

  static int? _asNullableInt(Object? value) {
    if (value == null) {
      return null;
    }
    if (value is int) {
      return value;
    }
    if (value is num) {
      return value.toInt();
    }
    return int.tryParse(value.toString());
  }

  static double _asDouble(Object? value, double fallback) {
    if (value == null) {
      return fallback;
    }
    if (value is num) {
      return value.toDouble();
    }
    return double.tryParse(value.toString()) ?? fallback;
  }

  static bool _asBool(Object? value, bool fallback) {
    if (value == null) {
      return fallback;
    }
    if (value is bool) {
      return value;
    }
    final text = value.toString().toLowerCase();
    if (text == 'true') {
      return true;
    }
    if (text == 'false') {
      return false;
    }
    return fallback;
  }
}
