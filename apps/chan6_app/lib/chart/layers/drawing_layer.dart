import 'package:flutter/material.dart';

import '../core/chart_layer.dart';
import '../core/chart_models.dart';
import '../core/coordinate_system.dart';

class DrawingLayer extends ChartLayer {
  const DrawingLayer() : super(id: 'drawing');

  @override
  void paint(
    Canvas canvas,
    Size size,
    ChartState state,
    CoordinateSystem coordinateSystem,
  ) {
    if (state.drawings.isEmpty || state.kline.isEmpty) {
      return;
    }

    final paint = Paint()
      ..color = const Color(0xffffcc80)
      ..strokeWidth = 1.4
      ..style = PaintingStyle.stroke;

    for (final drawing in state.drawings) {
      final startIndex = _barIdToIndex(state, drawing.start.barId);
      final endIndex = _barIdToIndex(state, drawing.end.barId);

      if (startIndex == null || endIndex == null) {
        continue;
      }

      final start = Offset(
        coordinateSystem.indexToX(startIndex),
        coordinateSystem.priceToY(drawing.start.price),
      );

      final end = Offset(
        coordinateSystem.indexToX(endIndex),
        coordinateSystem.priceToY(drawing.end.price),
      );

      canvas.drawLine(start, end, paint);
    }
  }

  int? _barIdToIndex(ChartState state, int barId) {
    for (var i = 0; i < state.kline.length; i++) {
      if (state.kline[i].barId == barId) {
        return i;
      }
    }
    return null;
  }
}
