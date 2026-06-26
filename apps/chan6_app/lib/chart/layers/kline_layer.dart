import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../core/chart_layer.dart';
import '../core/chart_models.dart';
import '../core/coordinate_system.dart';

class KLineLayer extends ChartLayer {
  const KLineLayer() : super(id: 'kline');

  @override
  void paint(
    Canvas canvas,
    Size size,
    ChartState state,
    CoordinateSystem coordinateSystem,
  ) {
    final rect = coordinateSystem.chartRect;

    if (state.kline.isEmpty) {
      _paintEmpty(canvas, rect);
      return;
    }

    final maxIndex = math.max(0, state.kline.length - 1);
    final start = state.viewport.startIndex.clamp(0, maxIndex).toInt();
    final end = state.viewport.endIndex.clamp(0, maxIndex).toInt();

    if (end < start) {
      _paintEmpty(canvas, rect);
      return;
    }

    final slotWidth = rect.width / math.max(1, end - start + 1);

    for (var i = start; i <= end; i++) {
      if (i < 0 || i >= state.kline.length) {
        continue;
      }

      final bar = state.kline[i];
      final x = coordinateSystem.indexToX(i);
      final openY = coordinateSystem.priceToY(bar.open);
      final closeY = coordinateSystem.priceToY(bar.close);
      final highY = coordinateSystem.priceToY(bar.high);
      final lowY = coordinateSystem.priceToY(bar.low);

      final rising = bar.close >= bar.open;
      final color = rising ? const Color(0xffef5350) : const Color(0xff26a69a);

      final wickPaint = Paint()
        ..color = color
        ..strokeWidth = 1;

      final bodyPaint = Paint()
        ..color = color.withValues(alpha: rising ? 0.25 : 0.85)
        ..style = PaintingStyle.fill;

      canvas.drawLine(
        Offset(x, highY),
        Offset(x, lowY),
        wickPaint,
      );

      final bodyWidth = math.max(2.0, slotWidth * 0.58);
      final bodyTop = math.min(openY, closeY);
      final bodyBottom = math.max(openY, closeY);
      final bodyHeight = math.max(1.0, bodyBottom - bodyTop);

      canvas.drawRect(
        Rect.fromLTWH(
          x - bodyWidth / 2,
          bodyTop,
          bodyWidth,
          bodyHeight,
        ),
        bodyPaint,
      );
    }
  }

  void _paintEmpty(Canvas canvas, Rect rect) {
    final painter = TextPainter(
      text: const TextSpan(
        text: 'No kline data',
        style: TextStyle(
          color: Color(0xff78909c),
          fontSize: 12,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: rect.width);

    painter.paint(
      canvas,
      Offset(rect.left + 8, rect.top + 8),
    );
  }
}
