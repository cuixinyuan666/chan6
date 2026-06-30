import 'package:flutter/material.dart';

import '../core/chart_layer.dart';
import '../core/chart_models.dart';
import '../core/coordinate_system.dart';

class BiLineLayer extends ChartLayer {
  const BiLineLayer() : super(id: 'bi_line');

  @override
  void paint(
    Canvas canvas,
    Size size,
    ChartState state,
    CoordinateSystem coordinateSystem,
  ) {
    if (state.kline.isEmpty || state.biLines.isEmpty) {
      return;
    }

    final barIdToIndex = <int, int>{};
    for (var i = 0; i < state.kline.length; i++) {
      barIdToIndex[state.kline[i].barId] = i;
    }

    final upPaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.0
      ..color = const Color(0xffef5350);

    final downPaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.0
      ..color = const Color(0xff26a69a);

    final pendingPaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2
      ..color = const Color(0x99ffffff);

    for (final bi in state.biLines) {
      final startIndex = barIdToIndex[bi.startBarId];
      final endIndex = barIdToIndex[bi.endBarId];

      if (startIndex == null || endIndex == null) {
        continue;
      }

      if (endIndex < state.viewport.startIndex ||
          startIndex > state.viewport.endIndex) {
        continue;
      }

      final start = Offset(
        coordinateSystem.indexToX(startIndex),
        coordinateSystem.priceToY(bi.startPrice),
      );

      final end = Offset(
        coordinateSystem.indexToX(endIndex),
        coordinateSystem.priceToY(bi.endPrice),
      );

      final paint = bi.confirmed
          ? (bi.direction == 'up' ? upPaint : downPaint)
          : pendingPaint;

      canvas.drawLine(start, end, paint);
    }
  }
}
