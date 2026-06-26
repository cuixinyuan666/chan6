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

    final viewport = _buildInitialViewport(kline);

    final meta = ChartMeta(
      schemaVersion: _asInt(metaMap['schema_version'], 1),
      query: _asString(metaMap['query'], _asString(data['query'], 'unknown')),
      symbol: symbol,
      klineScope: _asString(metaMap['kline_scope'], 'unknown'),
      chipScope: _asString(metaMap['chip_scope'] ?? data['chip_scope'], 'unknown'),
      chipBarId: _asNullableInt(metaMap['chip_bar_id'] ?? data['chip_bar_id']),
      chipTruncated: _asBool(metaMap['chip_truncated'] ?? data['chip_truncated'], false),
      chipTop: _asInt(metaMap['chip_top'] ?? data['chip_top'], 0),
      targetBarId: _asNullableInt(metaMap['target_bar_id'] ?? data['target_bar_id']),
      targetIndex: _asNullableInt(metaMap['target_index'] ?? data['target_index']),
      klineCount: _asInt(metaMap['kline_count'] ?? data['kline_count'], kline.length),
      offset: _asNullableInt(metaMap['offset'] ?? data['offset']),
      limit: _asNullableInt(metaMap['limit'] ?? data['limit']),
      day: _asNullableInt(metaMap['day'] ?? data['day']),
      minute: _asNullableInt(metaMap['minute'] ?? data['minute']),
      before: _asNullableInt(metaMap['before'] ?? data['before']),
      after: _asNullableInt(metaMap['after'] ?? data['after']),
      windowOffset: _asNullableInt(metaMap['window_offset'] ?? data['window_offset']),
      windowLimit: _asNullableInt(metaMap['window_limit'] ?? data['window_limit']),
    );

    return ChartState(
      symbol: symbol,
      kline: kline,
      chip: chip,
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

  static ChartViewport _buildInitialViewport(List<KLinePoint> kline) {
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
