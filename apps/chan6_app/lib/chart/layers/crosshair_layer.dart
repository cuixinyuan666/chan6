import 'package:flutter/material.dart';

import '../core/chart_layer.dart';
import '../core/chart_models.dart';
import '../core/coordinate_system.dart';

class CrosshairLayer extends ChartLayer {
  const CrosshairLayer() : super(id: 'crosshair');

  @override
  void paint(
    Canvas canvas,
    Size size,
    ChartState state,
    CoordinateSystem coordinateSystem,
  ) {
    final crosshair = state.crosshair;
    if (!crosshair.visible) {
      return;
    }

    final paint = Paint()
      ..color = const Color(0xffffffff).withValues(alpha: 0.45)
      ..strokeWidth = 1;

    canvas.drawLine(
      Offset(crosshair.x, coordinateSystem.chartRect.top),
      Offset(crosshair.x, coordinateSystem.chartRect.bottom),
      paint,
    );

    canvas.drawLine(
      Offset(coordinateSystem.chartRect.left, crosshair.y),
      Offset(coordinateSystem.chartRect.right, crosshair.y),
      paint,
    );
  }
}
