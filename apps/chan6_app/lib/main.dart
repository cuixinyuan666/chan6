import 'dart:async';
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
  static const _dbPath = 'data/cache/chan6_002003_fixed.db';
  static const _symbol = '002003';

  final _fallback = const SampleChartQueryRepository();

  CliChartQueryRepository? _cli;
  ChartState? _state;
  String _source = 'loading';
  String _message = '正在加载初始图表...';

  Timer? _hoverDebounce;
  int? _pendingBarId;
  int? _loadedChipBarId;
  int _requestSeq = 0;
  bool _loadingChip = false;

  @override
  void initState() {
    super.initState();
    _loadInitialChart();
  }

  @override
  void dispose() {
    _hoverDebounce?.cancel();
    super.dispose();
  }

  Future<void> _loadInitialChart() async {
    try {
      final repoRoot = CliChartQueryRepository.findRepoRoot();
      final dbFile = File('${repoRoot.path}${Platform.pathSeparator}$_dbPath');

      if (!dbFile.existsSync()) {
        final state = await _fallback.queryChart(
          dbPath: _dbPath,
          symbol: _symbol,
        );

        if (!mounted) {
          return;
        }

        setState(() {
          _state = state;
          _source = 'sample';
          _message = '本地数据库不存在，使用 sample 数据：$_dbPath';
        });
        return;
      }

      final cli = CliChartQueryRepository(repoRoot: repoRoot);
      final state = await cli.queryChart(
        dbPath: _dbPath,
        symbol: _symbol,
        offset: 1126500,
        limit: 160,
        top: 0,
      );

      if (!mounted) {
        return;
      }

      setState(() {
        _cli = cli;
        _state = state;
        _source = 'chan6_cli';
        _message = '已加载 chan6_cli query-chart 真实数据；移动鼠标可联动 query-chart-at 刷新筹码';
      });
    } catch (error) {
      final state = await _fallback.queryChart(
        dbPath: _dbPath,
        symbol: _symbol,
      );

      if (!mounted) {
        return;
      }

      setState(() {
        _state = state;
        _source = 'sample';
        _message = 'CLI 加载失败，使用 sample 数据：$error';
      });
    }
  }

  void _handleHoverBarChanged(KLinePoint bar) {
    final cli = _cli;
    if (cli == null) {
      return;
    }

    if (_loadedChipBarId == bar.barId || _pendingBarId == bar.barId) {
      return;
    }

    _pendingBarId = bar.barId;
    _hoverDebounce?.cancel();
    _hoverDebounce = Timer(
      const Duration(milliseconds: 220),
      () => _loadChipAt(cli, bar),
    );
  }

  Future<void> _loadChipAt(
    CliChartQueryRepository cli,
    KLinePoint bar,
  ) async {
    final seq = ++_requestSeq;

    setState(() {
      _loadingChip = true;
      _message =
          '正在查询 ${bar.tradingDay} ${bar.minute} 对应的完整历史筹码，bar_id=${bar.barId}';
    });

    try {
      final targetState = await cli.queryChartAt(
        dbPath: _dbPath,
        symbol: bar.symbol,
        day: bar.tradingDay,
        minute: bar.minute,
        before: 60,
        after: 60,
        top: 0,
      );

      if (!mounted || seq != _requestSeq || targetState == null) {
        return;
      }

      setState(() {
        _loadedChipBarId = bar.barId;
        _state = _state!.copyWith(
          chip: targetState.chip,
          meta: targetState.meta,
        );
        _loadingChip = false;
        _source = 'chan6_cli/query-chart-at';
        _message =
            '已刷新筹码：bar_id=${bar.barId}, ${bar.tradingDay} ${bar.minute}, chip_count=${targetState.chip.length}';
      });
    } catch (error) {
      if (!mounted || seq != _requestSeq) {
        return;
      }

      setState(() {
        _loadingChip = false;
        _message = 'query-chart-at 失败，保留当前筹码：$error';
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final state = _state;

    if (state == null) {
      return const Scaffold(
        body: Center(
          child: CircularProgressIndicator(),
        ),
      );
    }

    return Scaffold(
      body: Stack(
        children: [
          ChartShell(
            initialState: state,
            onHoverBarChanged: _handleHoverBarChanged,
          ),
          Positioned(
            left: 12,
            right: 12,
            bottom: 12,
            child: _StatusBar(
              source: _source,
              message: _message,
              loadingChip: _loadingChip,
            ),
          ),
        ],
      ),
    );
  }
}

class _StatusBar extends StatelessWidget {
  const _StatusBar({
    required this.source,
    required this.message,
    required this.loadingChip,
  });

  final String source;
  final String message;
  final bool loadingChip;

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
        child: Row(
          children: [
            if (loadingChip) ...[
              const SizedBox(
                width: 14,
                height: 14,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
              const SizedBox(width: 8),
            ],
            Expanded(
              child: Text(
                'source=$source | $message',
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(
                  fontSize: 12,
                  color: Color(0xffcfd8dc),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
