import 'package:flutter/material.dart';

import '../core/chart_layer.dart';
import '../core/chart_models.dart';
import '../core/coordinate_system.dart';

class CrosshairLayer extends ChartLayer {
  const CrosshairLayer() : super(id: 'crosshair');

  @override
  void paint(Canvas canvas, Size size, ChartState state, CoordinateSystem coord) {
    final crosshair = state.crosshair;
    if (!crosshair.visible || crosshair.index == null || crosshair.price == null) {
      return;
    }

    final x = coord.indexToX(crosshair.index!);
    final y = coord.priceToY(crosshair.price!);

    final paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..color = const Color(0xffffffff).withValues(alpha: 0.45);

    canvas.drawLine(
      Offset(x, coord.chartRect.top),
      Offset(x, coord.chartRect.bottom),
      paint,
    );

    canvas.drawLine(
      Offset(coord.chartRect.left, y),
      Offset(coord.chartRect.right, y),
      paint,
    );
  }
}
