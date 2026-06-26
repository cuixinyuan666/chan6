import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../core/chart_layer.dart';
import '../core/chart_models.dart';
import '../core/coordinate_system.dart';

class KLineLayer extends ChartLayer {
  const KLineLayer() : super(id: 'kline');

  @override
  void paint(Canvas canvas, Size size, ChartState state, CoordinateSystem coord) {
    final borderPaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;

    final bodyPaint = Paint()
      ..style = PaintingStyle.fill;

    final maxIndex = math.max(0, state.kline.length - 1);
    final start = state.viewport.startIndex.clamp(0, maxIndex).toInt();
    final end = state.viewport.endIndex.clamp(0, maxIndex).toInt();
    final bodyWidth = math.max(1.0, coord.candleSlotWidth * 0.6);

    for (var i = start; i <= end; i++) {
      final bar = state.kline[i];
      final x = coord.indexToX(i);
      final openY = coord.priceToY(bar.open);
      final closeY = coord.priceToY(bar.close);
      final highY = coord.priceToY(bar.high);
      final lowY = coord.priceToY(bar.low);

      final rising = bar.close >= bar.open;
      final color = rising ? const Color(0xffef5350) : const Color(0xff26a69a);
      borderPaint.color = color;
      bodyPaint.color = color.withValues(alpha: rising ? 0.25 : 0.85);

      canvas.drawLine(Offset(x, highY), Offset(x, lowY), borderPaint);

      final top = math.min(openY, closeY);
      final bottom = math.max(openY, closeY);
      final rect = Rect.fromLTRB(
        x - bodyWidth / 2,
        top,
        x + bodyWidth / 2,
        bottom == top ? bottom + 1 : bottom,
      );

      canvas.drawRect(rect, bodyPaint);
      canvas.drawRect(rect, borderPaint);
    }
  }
}
