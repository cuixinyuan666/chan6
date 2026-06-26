import '../core/chart_models.dart';

abstract class ChartQueryRepository {
  Future<ChartState> queryChart({
    required String dbPath,
    required String symbol,
    int offset = 0,
    int limit = 300,
    int top = 0,
    int? chipBarId,
  });

  Future<ChartState?> queryChartAt({
    required String dbPath,
    required String symbol,
    required int day,
    required int minute,
    int before = 120,
    int after = 120,
    int top = 0,
  });
}
