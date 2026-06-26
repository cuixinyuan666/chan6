import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

import 'core/chart_layer.dart';
import 'core/chart_models.dart';
import 'core/coordinate_system.dart';
import 'core/layer_manager.dart';
import 'layers/chip_layer.dart';
import 'layers/crosshair_layer.dart';
import 'layers/drawing_layer.dart';
import 'layers/kline_layer.dart';

class ChartShell extends StatefulWidget {
  const ChartShell({
    super.key,
    required this.initialState,
    this.onHoverBarChanged,
  });

  final ChartState initialState;
  final ValueChanged<KLinePoint>? onHoverBarChanged;

  @override
  State<ChartShell> createState() => _ChartShellState();
}

class _ChartShellState extends State<ChartShell> {
  late ChartState _state = widget.initialState;
  int? _lastHoverBarId;

  final LayerManager _layerManager = LayerManager(
    <ChartLayer>[
      KLineLayer(),
      DrawingLayer(),
      ChipLayer(),
      CrosshairLayer(),
    ],
  );

  @override
  void didUpdateWidget(covariant ChartShell oldWidget) {
    super.didUpdateWidget(oldWidget);

    if (oldWidget.initialState != widget.initialState) {
      _state = widget.initialState.copyWith(
        crosshair: _state.crosshair,
      );
    }
  }

  void _handleHover(PointerHoverEvent event, CoordinateSystem coordinateSystem) {
    final local = event.localPosition;
    final chartRect = coordinateSystem.chartRect;

    if (!chartRect.contains(local) || _state.kline.isEmpty) {
      setState(() {
        _state = _state.copyWith(crosshair: CrosshairState.hidden);
      });
      return;
    }

    final index = coordinateSystem.xToIndex(local.dx);
    if (index < 0 || index >= _state.kline.length) {
      setState(() {
        _state = _state.copyWith(crosshair: CrosshairState.hidden);
      });
      return;
    }

    final price = coordinateSystem.yToPrice(local.dy);
    final bar = _state.kline[index];

    setState(() {
      _state = _state.copyWith(
        crosshair: CrosshairState(
          visible: true,
          x: coordinateSystem.indexToX(index),
          y: local.dy,
          index: index,
          price: price,
        ),
      );
    });

    if (_lastHoverBarId != bar.barId) {
      _lastHoverBarId = bar.barId;
      widget.onHoverBarChanged?.call(bar);
    }
  }

  void _handleExit(PointerExitEvent event) {
    _lastHoverBarId = null;
    setState(() {
      _state = _state.copyWith(crosshair: CrosshairState.hidden);
    });
  }

  @override
  Widget build(BuildContext context) {
    return ColoredBox(
      color: const Color(0xff11151c),
      child: LayoutBuilder(
        builder: (context, constraints) {
          final size = Size(
            constraints.maxWidth,
            constraints.maxHeight,
          );

          final chartRect = Rect.fromLTWH(
            12,
            44,
            size.width * 0.76 - 18,
            size.height - 72,
          );

          final chipRect = Rect.fromLTWH(
            chartRect.right + 12,
            chartRect.top,
            size.width - chartRect.right - 24,
            chartRect.height,
          );

          final coordinateSystem = CoordinateSystem(
            chartRect: chartRect,
            chipRect: chipRect,
            viewport: _state.viewport,
            klineLength: _state.kline.length,
          );

          return MouseRegion(
            onHover: (event) => _handleHover(event, coordinateSystem),
            onExit: _handleExit,
            child: CustomPaint(
              size: size,
              painter: _ChartPainter(
                state: _state,
                coordinateSystem: coordinateSystem,
                layerManager: _layerManager,
              ),
            ),
          );
        },
      ),
    );
  }
}

class _ChartPainter extends CustomPainter {
  const _ChartPainter({
    required this.state,
    required this.coordinateSystem,
    required this.layerManager,
  });

  final ChartState state;
  final CoordinateSystem coordinateSystem;
  final LayerManager layerManager;

  @override
  void paint(Canvas canvas, Size size) {
    final background = Paint()..color = const Color(0xff11151c);
    canvas.drawRect(Offset.zero & size, background);

    final framePaint = Paint()
      ..color = const Color(0xff2a3442)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;

    canvas.drawRect(coordinateSystem.chartRect, framePaint);
    canvas.drawRect(coordinateSystem.chipRect, framePaint);

    final titlePainter = TextPainter(
      text: TextSpan(
        text:
            'Chan6 ${state.symbol} | ${state.meta.query} | chip=${state.meta.chipScope} | chip_bar=${state.meta.chipBarId ?? '-'}',
        style: const TextStyle(
          color: Color(0xffcfd8dc),
          fontSize: 13,
        ),
      ),
      textDirection: TextDirection.ltr,
      maxLines: 1,
    )..layout(maxWidth: size.width - 24);

    titlePainter.paint(canvas, const Offset(12, 14));

    layerManager.paint(
      canvas,
      size,
      state,
      coordinateSystem,
    );
  }

  @override
  bool shouldRepaint(covariant _ChartPainter oldDelegate) {
    return oldDelegate.state != state ||
        oldDelegate.coordinateSystem.chartRect != coordinateSystem.chartRect ||
        oldDelegate.coordinateSystem.chipRect != coordinateSystem.chipRect;
  }
}
