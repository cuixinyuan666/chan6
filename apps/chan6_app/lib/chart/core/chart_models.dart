class KLinePoint {
  const KLinePoint({
    required this.symbol,
    required this.barId,
    required this.tradingDay,
    required this.minute,
    required this.open,
    required this.high,
    required this.low,
    required this.close,
    required this.volume,
    required this.amount,
  });

  final String symbol;
  final int barId;
  final int tradingDay;
  final int minute;
  final double open;
  final double high;
  final double low;
  final double close;
  final double volume;
  final double amount;
}

class ChipLevel {
  const ChipLevel({
    required this.priceTick,
    required this.price,
    required this.volume,
    required this.amount,
    required this.tradeCount,
  });

  final int priceTick;
  final double price;
  final double volume;
  final double amount;
  final int tradeCount;
}

class ChartMeta {
  const ChartMeta({
    required this.schemaVersion,
    required this.query,
    required this.symbol,
    required this.klineScope,
    required this.chipScope,
    required this.chipBarId,
    required this.chipTruncated,
    required this.chipTop,
  });

  final int schemaVersion;
  final String query;
  final String symbol;
  final String klineScope;
  final String chipScope;
  final int? chipBarId;
  final bool chipTruncated;
  final int chipTop;
}

class ChartViewport {
  const ChartViewport({
    required this.startIndex,
    required this.endIndex,
    required this.minPrice,
    required this.maxPrice,
  });

  final int startIndex;
  final int endIndex;
  final double minPrice;
  final double maxPrice;

  ChartViewport copyWith({
    int? startIndex,
    int? endIndex,
    double? minPrice,
    double? maxPrice,
  }) {
    return ChartViewport(
      startIndex: startIndex ?? this.startIndex,
      endIndex: endIndex ?? this.endIndex,
      minPrice: minPrice ?? this.minPrice,
      maxPrice: maxPrice ?? this.maxPrice,
    );
  }
}

class CrosshairState {
  const CrosshairState({
    required this.visible,
    required this.index,
    required this.price,
  });

  final bool visible;
  final int? index;
  final double? price;

  static const hidden = CrosshairState(
    visible: false,
    index: null,
    price: null,
  );
}

class DrawingObject {
  const DrawingObject({
    required this.id,
    required this.type,
    required this.points,
    this.visible = true,
    this.locked = false,
  });

  final String id;
  final String type;
  final List<ChartAnchor> points;
  final bool visible;
  final bool locked;
}

class ChartAnchor {
  const ChartAnchor({
    required this.barId,
    required this.price,
  });

  final int barId;
  final double price;
}

class ChartState {
  const ChartState({
    required this.symbol,
    required this.kline,
    required this.chip,
    required this.meta,
    required this.viewport,
    required this.crosshair,
    required this.drawings,
  });

  final String symbol;
  final List<KLinePoint> kline;
  final List<ChipLevel> chip;
  final ChartMeta meta;
  final ChartViewport viewport;
  final CrosshairState crosshair;
  final List<DrawingObject> drawings;

  ChartState copyWith({
    String? symbol,
    List<KLinePoint>? kline,
    List<ChipLevel>? chip,
    ChartMeta? meta,
    ChartViewport? viewport,
    CrosshairState? crosshair,
    List<DrawingObject>? drawings,
  }) {
    return ChartState(
      symbol: symbol ?? this.symbol,
      kline: kline ?? this.kline,
      chip: chip ?? this.chip,
      meta: meta ?? this.meta,
      viewport: viewport ?? this.viewport,
      crosshair: crosshair ?? this.crosshair,
      drawings: drawings ?? this.drawings,
    );
  }

  static ChartState demo() {
    final bars = <KLinePoint>[];
    var price = 10.0;

    for (var i = 0; i < 160; i++) {
      final open = price;
      final close = open + ((i % 7) - 3) * 0.03;
      final high = open > close ? open + 0.08 : close + 0.08;
      final low = open < close ? open - 0.08 : close - 0.08;
      price = close;

      bars.add(
        KLinePoint(
          symbol: '002003',
          barId: i,
          tradingDay: 20260511,
          minute: 930 + i,
          open: open,
          high: high,
          low: low,
          close: close,
          volume: 1000 + i * 3,
          amount: (1000 + i * 3) * close,
        ),
      );
    }

    final chip = <ChipLevel>[
      for (var i = 0; i < 40; i++)
        ChipLevel(
          priceTick: 9500 + i * 10,
          price: 9.5 + i * 0.01,
          volume: 1000.0 + (i % 9) * 500,
          amount: (1000.0 + (i % 9) * 500) * (9.5 + i * 0.01),
          tradeCount: 10 + i,
        ),
    ];

    return ChartState(
      symbol: '002003',
      kline: bars,
      chip: chip,
      meta: const ChartMeta(
        schemaVersion: 1,
        query: 'demo',
        symbol: '002003',
        klineScope: 'demo_window',
        chipScope: 'full_history_to_target_bar',
        chipBarId: 80,
        chipTruncated: false,
        chipTop: 0,
      ),
      viewport: const ChartViewport(
        startIndex: 0,
        endIndex: 159,
        minPrice: 9.4,
        maxPrice: 10.4,
      ),
      crosshair: CrosshairState.hidden,
      drawings: const [],
    );
  }
}
