import 'dart:io';

import '../core/chart_models.dart';
import 'chart_query_parser.dart';
import 'chart_query_repository.dart';

class CliChartQueryRepository implements ChartQueryRepository {
  CliChartQueryRepository({
    Directory? repoRoot,
    this.cargoExecutable = 'cargo',
  }) : repoRoot = repoRoot ?? findRepoRoot();

  final Directory repoRoot;
  final String cargoExecutable;

  static Directory findRepoRoot() {
    var dir = Directory.current.absolute;

    for (var i = 0; i < 12; i++) {
      final cargoToml = File('${dir.path}${Platform.pathSeparator}Cargo.toml');
      final cliDir = Directory(
        '${dir.path}${Platform.pathSeparator}crates${Platform.pathSeparator}chan6_cli',
      );

      if (cargoToml.existsSync() && cliDir.existsSync()) {
        return dir;
      }

      final parent = dir.parent.absolute;
      if (parent.path == dir.path) {
        break;
      }
      dir = parent;
    }

    throw StateError(
      'Cannot find chan6 repo root from ${Directory.current.path}',
    );
  }

  @override
  Future<ChartState> queryChart({
    required String dbPath,
    required String symbol,
    int offset = 0,
    int limit = 300,
    int top = 0,
    int? chipBarId,
  }) async {
    final args = <String>[
      'run',
      '-p',
      'chan6_cli',
      '--',
      'query-chart',
      '--db',
      dbPath,
      '--symbol',
      symbol,
      '--offset',
      offset.toString(),
      '--limit',
      limit.toString(),
      '--top',
      top.toString(),
    ];

    if (chipBarId != null) {
      args.addAll(['--chip-bar-id', chipBarId.toString()]);
    }

    final raw = await _runCli(args);
    return ChartQueryParser.parseJsonString(raw);
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
    final raw = await _runCli(
      <String>[
        'run',
        '-p',
        'chan6_cli',
        '--',
        'query-chart-at',
        '--db',
        dbPath,
        '--symbol',
        symbol,
        '--day',
        day.toString(),
        '--minute',
        minute.toString(),
        '--before',
        before.toString(),
        '--after',
        after.toString(),
        '--top',
        top.toString(),
      ],
    );

    if (raw.trim() == 'null') {
      return null;
    }

    return ChartQueryParser.parseJsonString(raw);
  }

  Future<String> _runCli(List<String> args) async {
    final result = await Process.run(
      cargoExecutable,
      args,
      workingDirectory: repoRoot.path,
      runInShell: true,
    );

    if (result.exitCode != 0) {
      throw ProcessException(
        cargoExecutable,
        args,
        'chan6_cli failed with exitCode=${result.exitCode}\n'
        'stdout:\n${result.stdout}\n'
        'stderr:\n${result.stderr}',
        result.exitCode,
      );
    }

    final stdoutText = result.stdout.toString().trim();
    if (stdoutText.isEmpty) {
      throw StateError('chan6_cli returned empty stdout');
    }

    return stdoutText;
  }
}
