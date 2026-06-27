import 'package:flutter/material.dart';

import '../core/chart_layer.dart';
import '../core/chart_models.dart';
import '../core/coordinate_system.dart';

class FxLayer extends ChartLayer {
  const FxLayer() : super(id: 'fx');

  @override
  void paint(Canvas canvas, Size size, ChartState state, CoordinateSystem coordinateSystem) {
    if (state.kline.isEmpty || state.fxLines.isEmpty) return;

    final barIdToIndex = <int, int>{};
    for (var i = 0; i < state.kline.length; i++) {
      barIdToIndex[state.kline[i].barId] = i;
    }

    final points = <Offset>[];
    final visibleFx = <FxLinePoint, Offset>{};

    for (final fx in state.fxLines) {
      final index = barIdToIndex[fx.barId];
      if (index == null) continue;
      if (index < state.viewport.startIndex || index > state.viewport.endIndex) continue;
      final offset = Offset(
        coordinateSystem.indexToX(index),
        coordinateSystem.priceToY(fx.price),
      );
      points.add(offset);
      visibleFx[fx] = offset;
    }

    if (points.length >= 2) {
      final linePaint = Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..color = const Color(0xff64b5f6);
      final path = Path()..moveTo(points.first.dx, points.first.dy);
      for (var i = 1; i < points.length; i++) {
        path.lineTo(points[i].dx, points[i].dy);
      }
      canvas.drawPath(path, linePaint);
    }

    final topPaint = Paint()
      ..style = PaintingStyle.fill
      ..color = const Color(0xffffcc66);
    final bottomPaint = Paint()
      ..style = PaintingStyle.fill
      ..color = const Color(0xff66d9ef);
    final outlinePaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..color = const Color(0xff101820);

    for (final entry in visibleFx.entries) {
      final fx = entry.key;
      final offset = entry.value;
      canvas.drawCircle(offset, 4.2, fx.isTop ? topPaint : bottomPaint);
      canvas.drawCircle(offset, 4.2, outlinePaint);
    }
  }
}
