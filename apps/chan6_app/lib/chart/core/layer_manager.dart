import 'package:flutter/widgets.dart';

import 'chart_layer.dart';
import 'chart_models.dart';
import 'coordinate_system.dart';

class LayerManager {
  LayerManager(this.layers);

  final List<ChartLayer> layers;

  void paint(
    Canvas canvas,
    Size size,
    ChartState state,
    CoordinateSystem coord,
  ) {
    for (final layer in layers) {
      if (layer.visible) {
        layer.paint(canvas, size, state, coord);
      }
    }
  }
}
