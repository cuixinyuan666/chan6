import 'package:flutter/widgets.dart';

import 'chart_models.dart';
import 'coordinate_system.dart';

abstract class ChartLayer {
  const ChartLayer({
    required this.id,
    this.visible = true,
  });

  final String id;
  final bool visible;

  void paint(
    Canvas canvas,
    Size size,
    ChartState state,
    CoordinateSystem coord,
  );
}
