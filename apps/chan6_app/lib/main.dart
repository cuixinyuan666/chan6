import 'dart:io';

import 'package:flutter/material.dart';

import 'chart/chart_shell.dart';
import 'chart/core/chart_models.dart';
import 'chart/data/cli_chart_query_repository.dart';
import 'chart/data/sample_chart_query_repository.dart';

void main() {
  runApp(const Chan6App());
}

class Chan6App extends StatelessWidget {
  const Chan6App({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Chan6',
      debugShowCheckedModeBanner: false,
      theme: ThemeData.dark(useMaterial3: true),
      home: const ChartDemoPage(),
    );
  }
}

class ChartDemoPage extends StatefulWidget {
  const ChartDemoPage({super.key});

  @override
  State<ChartDemoPage> createState() => _ChartDemoPageState();
}

class _ChartDemoPageState extends State<ChartDemoPage> {
  late final Future<_LoadResult> _loadFuture = _loadInitialChart();

  static const _dbPath = 'data/cache/chan6_002003_fixed.db';
  static const _symbol = '002003';

  Future<_LoadResult> _loadInitialChart() async {
    final fallback = const SampleChartQueryRepository();

    try {
      final repoRoot = CliChartQueryRepository.findRepoRoot();
      final dbFile = File('${repoRoot.path}${Platform.pathSeparator}$_dbPath');

      if (!dbFile.existsSync()) {
        final state = await fallback.queryChart(
          dbPath: _dbPath,
          symbol: _symbol,
        );
        return _LoadResult(
          state: state,
          source: 'sample',
          message: '本地数据库不存在，使用 sample 数据：$_dbPath',
        );
      }

      final cli = CliChartQueryRepository(repoRoot: repoRoot);
      final state = await cli.queryChart(
        dbPath: _dbPath,
        symbol: _symbol,
        offset: 1126500,
        limit: 160,
        top: 0,
      );

      return _LoadResult(
        state: state,
        source: 'chan6_cli',
        message: '已加载 chan6_cli query-chart 真实数据',
      );
    } catch (error) {
      final state = await fallback.queryChart(
        dbPath: _dbPath,
        symbol: _symbol,
      );
      return _LoadResult(
        state: state,
        source: 'sample',
        message: 'CLI 加载失败，使用 sample 数据：$error',
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<_LoadResult>(
      future: _loadFuture,
      builder: (context, snapshot) {
        if (!snapshot.hasData) {
          return const Scaffold(
            body: Center(
              child: CircularProgressIndicator(),
            ),
          );
        }

        final result = snapshot.data!;

        return Scaffold(
          body: Stack(
            children: [
              ChartShell(initialState: result.state),
              Positioned(
                left: 12,
                right: 12,
                bottom: 12,
                child: _StatusBar(result: result),
              ),
            ],
          ),
        );
      },
    );
  }
}

class _StatusBar extends StatelessWidget {
  const _StatusBar({
    required this.result,
  });

  final _LoadResult result;

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: Colors.black.withValues(alpha: 0.55),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: Colors.white.withValues(alpha: 0.15),
        ),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        child: Text(
          'source=${result.source} | ${result.message}',
          maxLines: 2,
          overflow: TextOverflow.ellipsis,
          style: const TextStyle(
            fontSize: 12,
            color: Color(0xffcfd8dc),
          ),
        ),
      ),
    );
  }
}

class _LoadResult {
  const _LoadResult({
    required this.state,
    required this.source,
    required this.message,
  });

  final ChartState state;
  final String source;
  final String message;
}
