import 'dart:math' as math;

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

class MergedBox {
  const MergedBox({
    required this.index,
    required this.startBarId,
    required this.endBarId,
    required this.high,
    required this.low,
    required this.isMerged,
    required this.highBarId,
    required this.lowBarId,
    required this.calcHigh,
    required this.calcLow,
    required this.calcHighBarId,
    required this.calcLowBarId,
  });

  final int index;
  final int startBarId;
  final int endBarId;
  final double high;
  final double low;
  final bool isMerged;
  final int highBarId;
  final int lowBarId;
  final double calcHigh;
  final double calcLow;
  final int calcHighBarId;
  final int calcLowBarId;
}

class FxLinePoint {
  const FxLinePoint({
    required this.index,
    required this.kind,
    required this.mergedIndex,
    required this.barId,
    required this.price,
    required this.confirmed,
  });

  final int index;
  final String kind;
  final int mergedIndex;
  final int barId;
  final double price;
  final bool confirmed;

  bool get isTop => kind == 'top';
  bool get isBottom => kind == 'bottom';
}

class BiLinePoint {
  const BiLinePoint({
    required this.index,
    required this.direction,
    required this.startBarId,
    required this.startPrice,
    required this.endBarId,
    required this.endPrice,
    required this.confirmed,
  });

  final int index;
  final String direction;
  final int startBarId;
  final double startPrice;
  final int endBarId;
  final double endPrice;
  final bool confirmed;
}

class SegmentLinePoint {
  const SegmentLinePoint({
    required this.index,
    required this.n,
    required this.inputN,
    required this.direction,
    required this.startBiIndex,
    required this.endBiIndex,
    required this.startBarId,
    required this.startPrice,
    required this.endBarId,
    required this.endPrice,
    required this.confirmed,
    required this.reason,
  });

  final int index;
  final int n;
  final int inputN;
  final String direction;
  final int? startBiIndex;
  final int? endBiIndex;
  final int startBarId;
  final double startPrice;
  final int endBarId;
  final double endPrice;
  final bool confirmed;
  final String reason;
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
    this.targetBarId,
    this.targetIndex,
    this.klineCount,
    this.mergedCount,
    this.fxCount,
    this.biCount,
    this.segmentCount,
    this.offset,
    this.limit,
    this.day,
    this.minute,
    this.before,
    this.after,
    this.windowOffset,
    this.windowLimit,
  });

  final int schemaVersion;
  final String query;
  final String symbol;
  final String klineScope;
  final String chipScope;
  final int? chipBarId;
  final bool chipTruncated;
  final int chipTop;

  final int? targetBarId;
  final int? targetIndex;
  final int? klineCount;
  final int? mergedCount;
  final int? fxCount;
  final int? biCount;
  final int? segmentCount;
  final int? offset;
  final int? limit;
  final int? day;
  final int? minute;
  final int? before;
  final int? after;
  final int? windowOffset;
  final int? windowLimit;
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
}

class CrosshairState {
  const CrosshairState({
    required this.visible,
    required this.x,
    required this.y,
    required this.index,
    required this.price,
  });

  static const hidden = CrosshairState(
    visible: false,
    x: 0,
    y: 0,
    index: 0,
    price: 0,
  );

  final bool visible;
  final double x;
  final double y;
  final int index;
  final double price;
}

class ChartAnchor {
  const ChartAnchor({required this.barId, required this.price});

  final int barId;
  final double price;
}

class DrawingObject {
  const DrawingObject.line({
    required this.id,
    required this.start,
    required this.end,
  });

  final String id;
  final ChartAnchor start;
  final ChartAnchor end;
}

class ChartState {
  const ChartState({
    required this.symbol,
    required this.kline,
    required this.chip,
    required this.mergedBoxes,
    required this.fxLines,
    required this.biLines,
    required this.segmentLines,
    required this.meta,
    required this.viewport,
    required this.crosshair,
    required this.drawings,
  });

  final String symbol;
  final List<KLinePoint> kline;
  final List<ChipLevel> chip;
  final List<MergedBox> mergedBoxes;
  final List<FxLinePoint> fxLines;
  final List<BiLinePoint> biLines;
  final List<SegmentLinePoint> segmentLines;
  final ChartMeta meta;
  final ChartViewport viewport;
  final CrosshairState crosshair;
  final List<DrawingObject> drawings;

  ChartState copyWith({
    String? symbol,
    List<KLinePoint>? kline,
    List<ChipLevel>? chip,
    List<MergedBox>? mergedBoxes,
    List<FxLinePoint>? fxLines,
    List<BiLinePoint>? biLines,
    List<SegmentLinePoint>? segmentLines,
    ChartMeta? meta,
    ChartViewport? viewport,
    CrosshairState? crosshair,
    List<DrawingObject>? drawings,
  }) {
    return ChartState(
      symbol: symbol ?? this.symbol,
      kline: kline ?? this.kline,
      chip: chip ?? this.chip,
      mergedBoxes: mergedBoxes ?? this.mergedBoxes,
      fxLines: fxLines ?? this.fxLines,
      biLines: biLines ?? this.biLines,
      segmentLines: segmentLines ?? this.segmentLines,
      meta: meta ?? this.meta,
      viewport: viewport ?? this.viewport,
      crosshair: crosshair ?? this.crosshair,
      drawings: drawings ?? this.drawings,
    );
  }

  factory ChartState.demo() {
    final kline = <KLinePoint>[];
    var price = 10.0;

    for (var i = 0; i < 160; i++) {
      final wave = math.sin(i / 8.0) * 0.08;
      final open = price;
      final close = price + wave + (i % 7 - 3) * 0.01;
      final high = math.max(open, close) + 0.08 + (i % 5) * 0.01;
      final low = math.min(open, close) - 0.08 - (i % 3) * 0.01;

      kline.add(
        KLinePoint(
          symbol: 'DEMO',
          barId: i,
          tradingDay: 20260626,
          minute: 930 + i,
          open: open,
          high: high,
          low: low,
          close: close,
          volume: 1000 + i * 3,
          amount: (1000 + i * 3) * close,
        ),
      );

      price = close;
    }

    var minPrice = kline.first.low;
    var maxPrice = kline.first.high;
    for (final item in kline) {
      minPrice = math.min(minPrice, item.low);
      maxPrice = math.max(maxPrice, item.high);
    }

    final chip = <ChipLevel>[];
    for (var i = 0; i < 40; i++) {
      final p = minPrice + (maxPrice - minPrice) * i / 39;
      chip.add(
        ChipLevel(
          priceTick: (p * 1000).round(),
          price: p,
          volume: 1000 + math.sin(i / 5) * 600 + i * 30,
          amount: p * 1000,
          tradeCount: 10 + i,
        ),
      );
    }

    return ChartState(
      symbol: 'DEMO',
      kline: kline,
      chip: chip,
      mergedBoxes: const [],
      fxLines: const [],
      biLines: const [],
      segmentLines: const [],
      meta: const ChartMeta(
        schemaVersion: 1,
        query: 'demo',
        symbol: 'DEMO',
        klineScope: 'demo',
        chipScope: 'demo',
        chipBarId: null,
        chipTruncated: false,
        chipTop: 0,
      ),
      viewport: ChartViewport(
        startIndex: 0,
        endIndex: kline.length - 1,
        minPrice: minPrice,
        maxPrice: maxPrice,
      ),
      crosshair: CrosshairState.hidden,
      drawings: const [],
    );
  }
}
