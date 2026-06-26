import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../core/chart_layer.dart';
import '../core/chart_models.dart';
import '../core/coordinate_system.dart';

class ChipLayer extends ChartLayer {
  const ChipLayer() : super(id: 'chip');

  @override
  void paint(Canvas canvas, Size size, ChartState state, CoordinateSystem coord) {
    if (state.chip.isEmpty) {
      return;
    }

    final maxVolume = state.chip.fold<double>(
      0,
      (maxValue, level) => math.max(maxValue, level.volume),
    );

    if (maxVolume <= 0) {
      return;
    }

    final paint = Paint()
      ..style = PaintingStyle.fill
      ..color = const Color(0xff90caf9).withValues(alpha: 0.55);

    for (final level in state.chip) {
      final y = coord.chipPriceToY(level.price);
      if (y < coord.chipRect.top || y > coord.chipRect.bottom) {
        continue;
      }

      final width = coord.chipRect.width * (level.volume / maxVolume);
      final rect = Rect.fromLTWH(
        coord.chipRect.left,
        y - 2,
        width,
        4,
      );

      canvas.drawRect(rect, paint);
    }
  }
}
