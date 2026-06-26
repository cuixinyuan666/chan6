import 'package:flutter/material.dart';

import '../core/chart_layer.dart';
import '../core/chart_models.dart';
import '../core/coordinate_system.dart';

class DrawingLayer extends ChartLayer {
  const DrawingLayer() : super(id: 'drawing');

  @override
  void paint(Canvas canvas, Size size, ChartState state, CoordinateSystem coord) {
    final paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2
      ..color = const Color(0xffffca28);

    for (final drawing in state.drawings) {
      if (!drawing.visible || drawing.points.length < 2) {
        continue;
      }

      final p1 = drawing.points[0];
      final p2 = drawing.points[1];

      final i1 = state.kline.indexWhere((x) => x.barId == p1.barId);
      final i2 = state.kline.indexWhere((x) => x.barId == p2.barId);

      if (i1 < 0 || i2 < 0) {
        continue;
      }

      canvas.drawLine(
        Offset(coord.indexToX(i1), coord.priceToY(p1.price)),
        Offset(coord.indexToX(i2), coord.priceToY(p2.price)),
        paint,
      );
    }
  }
}
