import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../core/chart_layer.dart';
import '../core/chart_models.dart';
import '../core/coordinate_system.dart';

class ChipLayer extends ChartLayer {
  const ChipLayer() : super(id: 'chip');

  @override
  void paint(
    Canvas canvas,
    Size size,
    ChartState state,
    CoordinateSystem coordinateSystem,
  ) {
    final rect = coordinateSystem.chipRect;
    if (state.chip.isEmpty) {
      _paintEmpty(canvas, rect);
      return;
    }

    final maxVolume = state.chip.fold<double>(
      0,
      (maxValue, item) => math.max(maxValue, item.volume),
    );

    if (maxVolume <= 0) {
      _paintEmpty(canvas, rect);
      return;
    }

    final basePaint = Paint()
      ..color = const Color(0xff90caf9).withValues(alpha: 0.50)
      ..style = PaintingStyle.fill;

    final selectedPaint = Paint()
      ..color = const Color(0xffffcc80).withValues(alpha: 0.90)
      ..style = PaintingStyle.fill;

    final selectedBar = _findSelectedBar(state);
    final selectedPrice = selectedBar?.close;

    for (final level in state.chip) {
      final y = coordinateSystem.chipPriceToY(level.price);

      if (y < rect.top || y > rect.bottom) {
        continue;
      }

      final widthRatio = level.volume / maxVolume;
      final barWidth = math.max(1.0, rect.width * widthRatio);

      final isNearSelectedPrice = selectedPrice != null &&
          (level.price - selectedPrice).abs() <= _selectedPriceTolerance(state);

      final paint = isNearSelectedPrice ? selectedPaint : basePaint;

      canvas.drawRect(
        Rect.fromLTWH(
          rect.right - barWidth,
          y - 1.0,
          barWidth,
          2.0,
        ),
        paint,
      );
    }

    if (selectedBar != null) {
      _paintSelectedBarMarker(
        canvas: canvas,
        rect: rect,
        coordinateSystem: coordinateSystem,
        bar: selectedBar,
        meta: state.meta,
      );
    }

    _paintChipMeta(
      canvas: canvas,
      rect: rect,
      state: state,
      maxVolume: maxVolume,
    );
  }

  KLinePoint? _findSelectedBar(ChartState state) {
    final chipBarId = state.meta.chipBarId;
    if (chipBarId == null) {
      return null;
    }

    for (final bar in state.kline) {
      if (bar.barId == chipBarId) {
        return bar;
      }
    }

    return null;
  }

  double _selectedPriceTolerance(ChartState state) {
    if (state.kline.isEmpty) {
      return 0.005;
    }

    final range = math.max(
      0.000001,
      state.viewport.maxPrice - state.viewport.minPrice,
    );

    return math.max(0.005, range * 0.003);
  }

  void _paintSelectedBarMarker({
    required Canvas canvas,
    required Rect rect,
    required CoordinateSystem coordinateSystem,
    required KLinePoint bar,
    required ChartMeta meta,
  }) {
    final y = coordinateSystem.chipPriceToY(bar.close);

    if (y < rect.top || y > rect.bottom) {
      return;
    }

    final linePaint = Paint()
      ..color = const Color(0xffffcc80)
      ..strokeWidth = 1.2;

    canvas.drawLine(
      Offset(rect.left, y),
      Offset(rect.right, y),
      linePaint,
    );

    final label = 'bar=${bar.barId}  close=${bar.close.toStringAsFixed(2)}';
    final painter = TextPainter(
      text: TextSpan(
        text: label,
        style: const TextStyle(
          color: Color(0xffffcc80),
          fontSize: 11,
        ),
      ),
      textDirection: TextDirection.ltr,
      maxLines: 1,
    )..layout(maxWidth: rect.width - 8);

    final labelY = (y - 18).clamp(rect.top + 4, rect.bottom - 18).toDouble();

    painter.paint(
      canvas,
      Offset(rect.left + 4, labelY),
    );
  }

  void _paintChipMeta({
    required Canvas canvas,
    required Rect rect,
    required ChartState state,
    required double maxVolume,
  }) {
    final text =
        'chip_count=${state.chip.length}  chip_bar=${state.meta.chipBarId ?? '-'}  max=${maxVolume.toStringAsFixed(0)}';

    final painter = TextPainter(
      text: TextSpan(
        text: text,
        style: const TextStyle(
          color: Color(0xffb0bec5),
          fontSize: 10,
        ),
      ),
      textDirection: TextDirection.ltr,
      maxLines: 2,
    )..layout(maxWidth: rect.width - 8);

    painter.paint(
      canvas,
      Offset(rect.left + 4, rect.bottom - 28),
    );
  }

  void _paintEmpty(Canvas canvas, Rect rect) {
    final painter = TextPainter(
      text: const TextSpan(
        text: 'No chip data',
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
