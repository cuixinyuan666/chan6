import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

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
  });

  final ChartState initialState;

  @override
  State<ChartShell> createState() => _ChartShellState();
}

class _ChartShellState extends State<ChartShell> {
  late ChartState state;

  late final LayerManager layerManager = LayerManager(
    const [
      KLineLayer(),
      DrawingLayer(),
      ChipLayer(),
      CrosshairLayer(),
    ],
  );

  @override
  void initState() {
    super.initState();
    state = widget.initialState;
  }

  void _handleHover(PointerHoverEvent event, Size size) {
    final coord = _buildCoordinateSystem(size);
    final index = coord.xToIndex(event.localPosition.dx);
    final price = coord.yToPrice(event.localPosition.dy);

    setState(() {
      state = state.copyWith(
        crosshair: CrosshairState(
          visible: true,
          index: index,
          price: price,
        ),
      );
    });
  }

  void _handleExit(PointerExitEvent event) {
    setState(() {
      state = state.copyWith(crosshair: CrosshairState.hidden);
    });
  }

  CoordinateSystem _buildCoordinateSystem(Size size) {
    const leftPadding = 56.0;
    const topPadding = 24.0;
    const bottomPadding = 32.0;
    const chipWidth = 180.0;
    const gap = 16.0;

    final chartRect = Rect.fromLTWH(
      leftPadding,
      topPadding,
      size.width - leftPadding - chipWidth - gap - 24,
      size.height - topPadding - bottomPadding,
    );

    final chipRect = Rect.fromLTWH(
      chartRect.right + gap,
      chartRect.top,
      chipWidth,
      chartRect.height,
    );

    return CoordinateSystem(
      chartRect: chartRect,
      chipRect: chipRect,
      viewport: state.viewport,
      klineLength: state.kline.length,
    );
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final size = Size(
          constraints.maxWidth,
          constraints.maxHeight,
        );

        return MouseRegion(
          onHover: (event) => _handleHover(event, size),
          onExit: _handleExit,
          child: CustomPaint(
            size: size,
            painter: _ChartPainter(
              state: state,
              layerManager: layerManager,
              coordinateSystem: _buildCoordinateSystem(size),
            ),
          ),
        );
      },
    );
  }
}

class _ChartPainter extends CustomPainter {
  const _ChartPainter({
    required this.state,
    required this.layerManager,
    required this.coordinateSystem,
  });

  final ChartState state;
  final LayerManager layerManager;
  final CoordinateSystem coordinateSystem;

  @override
  void paint(Canvas canvas, Size size) {
    final backgroundPaint = Paint()
      ..style = PaintingStyle.fill
      ..color = const Color(0xff111318);

    canvas.drawRect(Offset.zero & size, backgroundPaint);

    final framePaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..color = const Color(0xff3a3f4b);

    canvas.drawRect(coordinateSystem.chartRect, framePaint);
    canvas.drawRect(coordinateSystem.chipRect, framePaint);

    layerManager.paint(canvas, size, state, coordinateSystem);

    final textPainter = TextPainter(
      text: TextSpan(
        text: '${state.symbol}  ${state.meta.chipScope}',
        style: const TextStyle(
          color: Color(0xffcfd8dc),
          fontSize: 12,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    textPainter.paint(
      canvas,
      Offset(coordinateSystem.chartRect.left, 4),
    );
  }

  @override
  bool shouldRepaint(covariant _ChartPainter oldDelegate) {
    return oldDelegate.state != state ||
        oldDelegate.coordinateSystem.chartRect != coordinateSystem.chartRect ||
        oldDelegate.coordinateSystem.chipRect != coordinateSystem.chipRect;
  }
}
