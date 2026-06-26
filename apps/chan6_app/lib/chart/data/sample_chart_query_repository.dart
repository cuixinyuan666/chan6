import '../core/chart_models.dart';
import 'chart_query_parser.dart';
import 'chart_query_repository.dart';
import 'chart_query_sample.dart';

class SampleChartQueryRepository implements ChartQueryRepository {
  const SampleChartQueryRepository();

  @override
  Future<ChartState> queryChart({
    required String dbPath,
    required String symbol,
    int offset = 0,
    int limit = 300,
    int top = 0,
    int? chipBarId,
  }) async {
    return ChartQueryParser.parseJsonString(chartQueryAtSampleJson);
  }

  @override
  Future<ChartState?> queryChartAt({
    required String dbPath,
    required String symbol,
    required int day,
    required int minute,
    int before = 120,
    int after = 120,
    int top = 0,
  }) async {
    return ChartQueryParser.parseJsonString(chartQueryAtSampleJson);
  }
}
