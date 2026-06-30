import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

import 'core/chart_layer.dart';
import 'core/chart_models.dart';
import 'core/coordinate_system.dart';
import 'core/layer_manager.dart';
import 'layers/bi_line_layer.dart';
import 'layers/chip_layer.dart';
import 'layers/crosshair_layer.dart';
import 'layers/drawing_layer.dart';
import 'layers/fx_layer.dart';
import 'layers/kline_layer.dart';
import 'layers/merged_box_layer.dart';

class ChartShell extends StatefulWidget {
  const ChartShell({
    super.key,
    required this.initialState,
    this.onHoverBarChanged,
    this.onPanWindow,
    this.onZoomWindow,
    this.drawLineMode = false,
  });

  final ChartState initialState;
  final ValueChanged<KLinePoint>? onHoverBarChanged;
  final ValueChanged<int>? onPanWindow;
  final ValueChanged<bool>? onZoomWindow;
  final bool drawLineMode;

  @override
  State<ChartShell> createState() => _ChartShellState();
}

class _ChartShellState extends State<ChartShell> {
  late ChartState _state = widget.initialState;
  int? _lastHoverBarId;
  ChartAnchor? _pendingLineStart;
  Offset? _panStartLocal;

  final LayerManager _layerManager = LayerManager(<ChartLayer>[
    KLineLayer(),
    MergedBoxLayer(),
    FxLayer(),
    BiLineLayer(),
    DrawingLayer(),
    ChipLayer(),
    CrosshairLayer(),
  ]);

  @override
  void didUpdateWidget(covariant ChartShell oldWidget) {
    super.didUpdateWidget(oldWidget);

    if (oldWidget.initialState != widget.initialState) {
      _state = widget.initialState.copyWith(
        crosshair: _state.crosshair,
        drawings: _state.drawings,
      );
    }

    if (oldWidget.drawLineMode && !widget.drawLineMode) {
      _pendingLineStart = null;
    }
  }

  void _handleHover(
    PointerHoverEvent event,
    CoordinateSystem coordinateSystem,
  ) {
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

    if (!widget.drawLineMode && _lastHoverBarId != bar.barId) {
      _lastHoverBarId = bar.barId;
      widget.onHoverBarChanged?.call(bar);
    }
  }

  void _handlePointerDown(
    PointerDownEvent event,
    CoordinateSystem coordinateSystem,
  ) {
    if (widget.drawLineMode) {
      _handleDrawLinePointerDown(event, coordinateSystem);
      return;
    }

    if (event.buttons == kPrimaryMouseButton &&
        coordinateSystem.chartRect.contains(event.localPosition)) {
      _panStartLocal = event.localPosition;
    }
  }

  void _handlePointerUp(
    PointerUpEvent event,
    CoordinateSystem coordinateSystem,
  ) {
    if (widget.drawLineMode) {
      return;
    }

    final start = _panStartLocal;
    _panStartLocal = null;

    if (start == null || _state.kline.isEmpty) {
      return;
    }

    final dx = event.localPosition.dx - start.dx;
    final slotWidth = coordinateSystem.chartRect.width / _state.kline.length;
    final deltaBars = (-dx / slotWidth).round();

    if (deltaBars.abs() >= 5) {
      widget.onPanWindow?.call(deltaBars);
    }
  }

  void _handlePointerCancel(PointerCancelEvent event) {
    _panStartLocal = null;
  }

  void _handlePointerSignal(PointerSignalEvent event) {
    if (widget.drawLineMode) {
      return;
    }

    if (event is PointerScrollEvent) {
      final zoomIn = event.scrollDelta.dy < 0;
      widget.onZoomWindow?.call(zoomIn);
    }
  }

  void _handleDrawLinePointerDown(
    PointerDownEvent event,
    CoordinateSystem coordinateSystem,
  ) {
    if (event.buttons == kSecondaryMouseButton) {
      setState(() {
        _pendingLineStart = null;
      });
      return;
    }

    if (event.buttons != kPrimaryMouseButton) {
      return;
    }

    final anchor = _buildAnchorFromLocalPosition(
      event.localPosition,
      coordinateSystem,
    );

    if (anchor == null) {
      return;
    }

    final start = _pendingLineStart;

    if (start == null) {
      setState(() {
        _pendingLineStart = anchor;
      });
      return;
    }

    final drawing = DrawingObject.line(
      id: 'line_${DateTime.now().microsecondsSinceEpoch}',
      start: start,
      end: anchor,
    );

    setState(() {
      _pendingLineStart = null;
      _state = _state.copyWith(
        drawings: <DrawingObject>[..._state.drawings, drawing],
      );
    });
  }

  ChartAnchor? _buildAnchorFromLocalPosition(
    Offset local,
    CoordinateSystem coordinateSystem,
  ) {
    if (!coordinateSystem.chartRect.contains(local) || _state.kline.isEmpty) {
      return null;
    }

    final index = coordinateSystem.xToIndex(local.dx);
    if (index < 0 || index >= _state.kline.length) {
      return null;
    }

    final bar = _state.kline[index];

    return ChartAnchor(
      barId: bar.barId,
      price: coordinateSystem.yToPrice(local.dy),
    );
  }

  void _handleExit(PointerExitEvent event) {
    _lastHoverBarId = null;
    _panStartLocal = null;
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
          final size = Size(constraints.maxWidth, constraints.maxHeight);

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
            child: Listener(
              behavior: HitTestBehavior.opaque,
              onPointerDown: (event) =>
                  _handlePointerDown(event, coordinateSystem),
              onPointerUp: (event) => _handlePointerUp(event, coordinateSystem),
              onPointerCancel: _handlePointerCancel,
              onPointerSignal: _handlePointerSignal,
              child: CustomPaint(
                size: size,
                painter: _ChartPainter(
                  state: _state,
                  coordinateSystem: coordinateSystem,
                  layerManager: _layerManager,
                  pendingLineStart: _pendingLineStart,
                ),
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
    required this.pendingLineStart,
  });

  final ChartState state;
  final CoordinateSystem coordinateSystem;
  final LayerManager layerManager;
  final ChartAnchor? pendingLineStart;

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
            'Chan6 ${state.symbol} | ${state.meta.query} | merged=${state.meta.mergedCount ?? state.mergedBoxes.length} | fx=${state.meta.fxCount ?? state.fxLines.length} | bi=${state.meta.biCount ?? state.biLines.length}',
        style: const TextStyle(color: Color(0xffcfd8dc), fontSize: 13),
      ),
      textDirection: TextDirection.ltr,
      maxLines: 1,
    )..layout(maxWidth: size.width - 24);

    titlePainter.paint(canvas, const Offset(12, 14));

    layerManager.paint(canvas, size, state, coordinateSystem);

    _paintPendingLine(canvas);
  }

  void _paintPendingLine(Canvas canvas) {
    final start = pendingLineStart;
    if (start == null || state.kline.isEmpty) {
      return;
    }

    final startIndex = _barIdToIndex(start.barId);
    if (startIndex == null) {
      return;
    }

    final startOffset = Offset(
      coordinateSystem.indexToX(startIndex),
      coordinateSystem.priceToY(start.price),
    );

    final paint = Paint()
      ..color = const Color(0xffffcc80)
      ..strokeWidth = 1.5;

    canvas.drawCircle(startOffset, 4, paint);

    if (state.crosshair.visible) {
      canvas.drawLine(
        startOffset,
        Offset(state.crosshair.x, state.crosshair.y),
        paint,
      );
    }
  }

  int? _barIdToIndex(int barId) {
    for (var i = 0; i < state.kline.length; i++) {
      if (state.kline[i].barId == barId) {
        return i;
      }
    }
    return null;
  }

  @override
  bool shouldRepaint(covariant _ChartPainter oldDelegate) {
    return oldDelegate.state != state ||
        oldDelegate.pendingLineStart != pendingLineStart ||
        oldDelegate.coordinateSystem.chartRect != coordinateSystem.chartRect ||
        oldDelegate.coordinateSystem.chipRect != coordinateSystem.chipRect;
  }
}
