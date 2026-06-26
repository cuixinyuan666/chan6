import 'dart:async';
import 'dart:io';
import 'dart:math' as math;

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
  Timer? _windowDebounce;

  int? _pendingBarId;
  int? _loadedChipBarId;
  int _requestSeq = 0;
  int _windowSeq = 0;

  int _windowOffset = 1126500;
  int _windowLimit = 160;

  bool _loadingChip = false;
  bool _loadingWindow = false;
  bool _drawLineMode = false;

  @override
  void initState() {
    super.initState();
    _loadInitialChart();
  }

  @override
  void dispose() {
    _hoverDebounce?.cancel();
    _windowDebounce?.cancel();
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
      _cli = cli;
      await _loadKlineWindow(
        offset: _windowOffset,
        limit: _windowLimit,
        reason: '初始加载',
      );
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

  Future<void> _loadKlineWindow({
    required int offset,
    required int limit,
    required String reason,
  }) async {
    final cli = _cli;
    if (cli == null) {
      return;
    }

    final previousOffset = _windowOffset;
    final previousLimit = _windowLimit;

    final safeOffset = math.max(0, offset);
    final safeLimit = limit.clamp(40, 1200).toInt();
    final seq = ++_windowSeq;

    setState(() {
      _loadingWindow = true;
      _windowOffset = safeOffset;
      _windowLimit = safeLimit;
      _source = 'chan6_cli/query-chart';
      _message =
          '$reason：正在加载窗口 offset=$_windowOffset, limit=$_windowLimit';
    });

    try {
      final state = await cli.queryChart(
        dbPath: _dbPath,
        symbol: _symbol,
        offset: safeOffset,
        limit: safeLimit,
        top: 0,
      );

      if (!mounted || seq != _windowSeq) {
        return;
      }

      if (state.kline.isEmpty) {
        setState(() {
          _loadingWindow = false;
          _loadingChip = false;
          _windowOffset = previousOffset;
          _windowLimit = previousLimit;
          _source = 'chan6_cli/query-chart';
          _message =
              '$reason：窗口超出数据范围，已保留当前窗口 offset=$previousOffset, limit=$previousLimit';
        });
        return;
      }

      setState(() {
        _state = state;
        _loadingWindow = false;
        _loadingChip = false;
        _pendingBarId = null;
        _loadedChipBarId = state.meta.chipBarId;
        _source = 'chan6_cli/query-chart';
        _message =
            '$reason：已加载K线窗口 offset=$_windowOffset, limit=$_windowLimit, kline_count=${state.kline.length}, chip_count=${state.chip.length}';
      });
    } catch (error) {
      if (!mounted || seq != _windowSeq) {
        return;
      }

      setState(() {
        _loadingWindow = false;
        _windowOffset = previousOffset;
        _windowLimit = previousLimit;
        _message =
            '$reason：query-chart 失败，已恢复当前窗口 offset=$previousOffset, limit=$previousLimit：$error';
      });
    }
  }

  void _scheduleKlineWindow({
    required int offset,
    required int limit,
    required String reason,
  }) {
    _windowDebounce?.cancel();
    _windowDebounce = Timer(
      const Duration(milliseconds: 180),
      () => _loadKlineWindow(
        offset: offset,
        limit: limit,
        reason: reason,
      ),
    );
  }

  void _handlePanWindow(int deltaBars) {
    if (_drawLineMode || _cli == null || deltaBars == 0) {
      return;
    }

    final nextOffset = math.max(0, _windowOffset + deltaBars);

    if (nextOffset == _windowOffset) {
      return;
    }

    _scheduleKlineWindow(
      offset: nextOffset,
      limit: _windowLimit,
      reason: '平移窗口 delta=$deltaBars',
    );
  }

  void _handleZoomWindow(bool zoomIn) {
    if (_drawLineMode || _cli == null) {
      return;
    }

    final center = _windowOffset + _windowLimit ~/ 2;
    final nextLimit = zoomIn
        ? math.max(40, (_windowLimit * 0.75).round())
        : math.min(1200, (_windowLimit * 1.33).round());

    if (nextLimit == _windowLimit) {
      return;
    }

    final nextOffset = math.max(0, center - nextLimit ~/ 2);

    _scheduleKlineWindow(
      offset: nextOffset,
      limit: nextLimit,
      reason: zoomIn ? '缩放窗口：放大' : '缩放窗口：缩小',
    );
  }

  void _moveWindowByButton(int deltaBars) {
    _handlePanWindow(deltaBars);
  }

  void _zoomWindowByButton(bool zoomIn) {
    _handleZoomWindow(zoomIn);
  }

  void _reloadWindow() {
    _scheduleKlineWindow(
      offset: _windowOffset,
      limit: _windowLimit,
      reason: '重载窗口',
    );
  }

  void _handleHoverBarChanged(KLinePoint bar) {
    final cli = _cli;
    if (cli == null || _loadingWindow) {
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
            onHoverBarChanged:
                _drawLineMode ? null : _handleHoverBarChanged,
            onPanWindow: _handlePanWindow,
            onZoomWindow: _handleZoomWindow,
            drawLineMode: _drawLineMode,
          ),
          Positioned(
            right: 12,
            top: 12,
            child: _ToolBar(
              drawLineMode: _drawLineMode,
              loadingWindow: _loadingWindow,
              onToggleDrawLine: () {
                setState(() {
                  _drawLineMode = !_drawLineMode;
                  _message = _drawLineMode
                      ? '画线模式：左键点击两个位置生成趋势线；右键取消当前起点'
                      : '已退出画线模式';
                });
              },
              onMoveLeft: () => _moveWindowByButton(-(_windowLimit ~/ 2)),
              onMoveRight: () => _moveWindowByButton(_windowLimit ~/ 2),
              onZoomIn: () => _zoomWindowByButton(true),
              onZoomOut: () => _zoomWindowByButton(false),
              onReload: _reloadWindow,
            ),
          ),
          Positioned(
            left: 12,
            right: 12,
            bottom: 12,
            child: _StatusBar(
              source: _source,
              message: _message,
              loadingChip: _loadingChip,
              loadingWindow: _loadingWindow,
              drawLineMode: _drawLineMode,
              windowOffset: _windowOffset,
              windowLimit: _windowLimit,
            ),
          ),
        ],
      ),
    );
  }
}

class _ToolBar extends StatelessWidget {
  const _ToolBar({
    required this.drawLineMode,
    required this.loadingWindow,
    required this.onToggleDrawLine,
    required this.onMoveLeft,
    required this.onMoveRight,
    required this.onZoomIn,
    required this.onZoomOut,
    required this.onReload,
  });

  final bool drawLineMode;
  final bool loadingWindow;
  final VoidCallback onToggleDrawLine;
  final VoidCallback onMoveLeft;
  final VoidCallback onMoveRight;
  final VoidCallback onZoomIn;
  final VoidCallback onZoomOut;
  final VoidCallback onReload;

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: Colors.black.withValues(alpha: 0.55),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Colors.white.withValues(alpha: 0.15),
        ),
      ),
      child: Padding(
        padding: const EdgeInsets.all(6),
        child: Wrap(
          spacing: 6,
          runSpacing: 6,
          alignment: WrapAlignment.end,
          children: [
            FilledButton.tonal(
              onPressed: loadingWindow ? null : onMoveLeft,
              child: const Text('左移'),
            ),
            FilledButton.tonal(
              onPressed: loadingWindow ? null : onMoveRight,
              child: const Text('右移'),
            ),
            FilledButton.tonal(
              onPressed: loadingWindow ? null : onZoomIn,
              child: const Text('放大'),
            ),
            FilledButton.tonal(
              onPressed: loadingWindow ? null : onZoomOut,
              child: const Text('缩小'),
            ),
            FilledButton.tonal(
              onPressed: loadingWindow ? null : onReload,
              child: const Text('重载'),
            ),
            FilledButton.tonal(
              onPressed: onToggleDrawLine,
              child: Text(drawLineMode ? '退出画线' : '画线'),
            ),
          ],
        ),
      ),
    );
  }
}

class _StatusBar extends StatelessWidget {
  const _StatusBar({
    required this.source,
    required this.message,
    required this.loadingChip,
    required this.loadingWindow,
    required this.drawLineMode,
    required this.windowOffset,
    required this.windowLimit,
  });

  final String source;
  final String message;
  final bool loadingChip;
  final bool loadingWindow;
  final bool drawLineMode;
  final int windowOffset;
  final int windowLimit;

  @override
  Widget build(BuildContext context) {
    final modeText = drawLineMode ? ' | 画线模式开启' : '';
    final windowText = ' | offset=$windowOffset limit=$windowLimit';

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
            if (loadingChip || loadingWindow) ...[
              const SizedBox(
                width: 14,
                height: 14,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
              const SizedBox(width: 8),
            ],
            Expanded(
              child: Text(
                'source=$source | $message$windowText$modeText',
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
