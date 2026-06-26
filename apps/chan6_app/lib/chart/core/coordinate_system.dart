import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'chart_models.dart';

class CoordinateSystem {
  CoordinateSystem({
    required this.chartRect,
    required this.chipRect,
    required this.viewport,
    required this.klineLength,
  });

  final Rect chartRect;
  final Rect chipRect;
  final ChartViewport viewport;
  final int klineLength;

  double get visibleCount {
    return math.max(1, viewport.endIndex - viewport.startIndex + 1).toDouble();
  }

  double get candleSlotWidth {
    return chartRect.width / visibleCount;
  }

  double indexToX(int index) {
    return chartRect.left + (index - viewport.startIndex + 0.5) * candleSlotWidth;
  }

  int xToIndex(double x) {
    final raw = ((x - chartRect.left) / candleSlotWidth).floor() + viewport.startIndex;
    return raw.clamp(0, math.max(0, klineLength - 1));
  }

  double priceToY(double price) {
    final range = math.max(0.000001, viewport.maxPrice - viewport.minPrice);
    final t = (viewport.maxPrice - price) / range;
    return chartRect.top + t * chartRect.height;
  }

  double yToPrice(double y) {
    final range = math.max(0.000001, viewport.maxPrice - viewport.minPrice);
    final t = (y - chartRect.top) / chartRect.height;
    return viewport.maxPrice - t * range;
  }

  double chipPriceToY(double price) {
    final range = math.max(0.000001, viewport.maxPrice - viewport.minPrice);
    final t = (viewport.maxPrice - price) / range;
    return chipRect.top + t * chipRect.height;
  }
}
