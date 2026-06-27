import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../core/chart_layer.dart';
import '../core/chart_models.dart';
import '../core/coordinate_system.dart';

class MergedBoxLayer extends ChartLayer {
  const MergedBoxLayer() : super(id: 'merged_box');

  @override
  void paint(Canvas canvas, Size size, ChartState state, CoordinateSystem coordinateSystem) {
    if (state.kline.isEmpty || state.mergedBoxes.isEmpty) return;

    final barIdToIndex = <int, int>{};
    for (var i = 0; i < state.kline.length; i++) {
      barIdToIndex[state.kline[i].barId] = i;
    }

    final maxIndex = math.max(0, state.kline.length - 1);
    final visibleStart = state.viewport.startIndex.clamp(0, maxIndex).toInt();
    final visibleEnd = state.viewport.endIndex.clamp(0, maxIndex).toInt();

    for (final box in state.mergedBoxes) {
      final startIndex = barIdToIndex[box.startBarId];
      final endIndex = barIdToIndex[box.endBarId];
      if (startIndex == null || endIndex == null) continue;
      if (endIndex < visibleStart || startIndex > visibleEnd) continue;

      final leftIndex = math.max(visibleStart, startIndex);
      final rightIndex = math.min(visibleEnd, endIndex);
      final left = coordinateSystem.indexToX(leftIndex) - coordinateSystem.candleSlotWidth * 0.47;
      final right = coordinateSystem.indexToX(rightIndex) + coordinateSystem.candleSlotWidth * 0.47;
      final top = coordinateSystem.priceToY(box.high);
      final bottom = coordinateSystem.priceToY(box.low);
      final rect = Rect.fromLTRB(left, top, right, bottom);

      final fillPaint = Paint()
        ..style = PaintingStyle.fill
        ..color = box.isMerged ? const Color(0x18ffd54f) : const Color(0x06ffd54f);
      final borderPaint = Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = box.isMerged ? 1.4 : 0.7
        ..color = box.isMerged ? const Color(0xeaffd54f) : const Color(0x40ffd54f);

      canvas.drawRect(rect, fillPaint);
      canvas.drawRect(rect, borderPaint);
    }
  }
}
