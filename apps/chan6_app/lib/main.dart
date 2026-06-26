import 'package:flutter/material.dart';

import 'chart/chart_shell.dart';
import 'chart/core/chart_models.dart';

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

class ChartDemoPage extends StatelessWidget {
  const ChartDemoPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: ChartShell(
        initialState: ChartState.demo(),
      ),
    );
  }
}
