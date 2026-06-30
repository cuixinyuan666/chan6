import 'package:flutter/material.dart';

import '../core/chart_layer.dart';
import '../core/chart_models.dart';
import '../core/coordinate_system.dart';

class SegmentLineLayer extends ChartLayer {
  const SegmentLineLayer() : super(id: 'segment_line');

  @override
  void paint(
    Canvas canvas,
    Size size,
    ChartState state,
    CoordinateSystem coordinateSystem,
  ) {
    if (state.kline.isEmpty || state.segmentLines.isEmpty) {
      return;
    }

    final barIdToIndex = <int, int>{};
    for (var i = 0; i < state.kline.length; i++) {
      barIdToIndex[state.kline[i].barId] = i;
    }

    final upPaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.8
      ..color = const Color(0xffff7043);

    final downPaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.8
      ..color = const Color(0xff00bcd4);

    final pendingPaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5
      ..color = const Color(0x99ffffff);

    for (final segment in state.segmentLines) {
      final startIndex = barIdToIndex[segment.startBarId];
      final endIndex = barIdToIndex[segment.endBarId];

      if (startIndex == null || endIndex == null) {
        continue;
      }

      if (endIndex < state.viewport.startIndex ||
          startIndex > state.viewport.endIndex) {
        continue;
      }

      final start = Offset(
        coordinateSystem.indexToX(startIndex),
        coordinateSystem.priceToY(segment.startPrice),
      );

      final end = Offset(
        coordinateSystem.indexToX(endIndex),
        coordinateSystem.priceToY(segment.endPrice),
      );

      final paint = segment.confirmed
          ? (segment.direction == 'up' ? upPaint : downPaint)
          : pendingPaint;

      canvas.drawLine(start, end, paint);
      _paintLabel(canvas, end, 'S${segment.index}', paint.color);
    }
  }

  void _paintLabel(Canvas canvas, Offset offset, String text, Color color) {
    final painter = TextPainter(
      text: TextSpan(
        text: text,
        style: TextStyle(
          color: color,
          fontSize: 10,
          fontWeight: FontWeight.w600,
        ),
      ),
      textDirection: TextDirection.ltr,
      maxLines: 1,
    )..layout(maxWidth: 48);

    painter.paint(canvas, Offset(offset.dx + 4, offset.dy - 14));
  }
}
