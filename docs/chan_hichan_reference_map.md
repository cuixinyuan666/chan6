# hichan 参考实现映射

本文档记录 `git@github.com:cuixinyuan666/chan_replay_app.git` 的 `hichan` 分支中与 Chan6 缠论实现相关的参考位置。

本文件只作为语义迁移索引。Chan6 的缠论计算仍必须使用 Rust 实现，不能把 Python 或 Flutter/Dart 判断逻辑作为 Chan6 的计算权威。

## 1. 参考工程入口

hichan 工程是 Flutter 项目，`pubspec.yaml` 描述为“缠论 K 线复盘 MVP，支持本地 CSV、逐 K 回放、分型、笔、中枢”。

关键入口：

```text
lib/main.dart
lib/app.dart
lib/ui/pages/root_page_four_way.dart
lib/ui/pages/kline_chart_page.dart
lib/ui/pages/s13_single_stock_replay_page.dart
```

其中：

- `main.dart` 负责 Flutter 启动和 Windows 窗口初始化。
- `app.dart` 负责主题和根页面。
- `root_page_four_way.dart` 注册 K线、设置、选股、回测、筹码等页面。
- `kline_chart_page.dart` 将 K线入口导向 S13 多级别复盘页面。
- `s13_single_stock_replay_page.dart` 是旧系统核心页面，包含多级别、节奏线、筹码、回测标记等 UI 状态。

## 2. hichan 模型层参考

### 2.1 ChanSnapshot

路径：

```text
lib/core/models/chan_snapshot.dart
```

语义：

```text
rawBars
mergedBars
fxs
bis
segs
recursiveSegLayers
recursiveSegBsps
recursiveSegBspCandidates
recursiveSegZss
zss
bsps
segZss
rhythmLines
rhythmHits
meta
```

Chan6 Rust 侧后续应按类似分层输出，但字段名和结构要稳定为 Rust JSON 契约。

### 2.2 RawBar

路径：

```text
lib/core/models/raw_bar.dart
```

语义字段：

```text
index
time
open
high
low
close
volume
chipTickBins
```

Chan6 已有 `KLine1m`，后续 ChanBar 应从 `KLine1m` 映射而来，并使用 `bar_id` 替代旧项目中的 `rawIndex/index` 作为业务锚点。

### 2.3 MergedBar

路径：

```text
lib/core/models/merged_bar.dart
```

语义字段：

```text
index
startRawIndex
endRawIndex
highRawIndex
lowRawIndex
time
highTime
lowTime
open
high
low
close
volume
```

Chan6 Rust 侧对应包含关系处理后的合并 K线。注意：旧项目使用 rawIndex，Chan6 必须输出 bar_id。

### 2.4 FX

路径：

```text
lib/core/models/fx.dart
```

语义字段：

```text
FxType.top
FxType.bottom
index
rawIndex
time
type
price
left
center
right
confirmed
```

Chan6 Rust 侧对应：

```text
ChanFxKind::Top
ChanFxKind::Bottom
bar_id
price
confirmed
```

### 2.5 BI

路径：

```text
lib/core/models/bi.dart
```

语义字段：

```text
BiDirection.up
BiDirection.down
index
start
end
direction
prevIndex
nextIndex
parentSegIndex
parentSegDirection
parentSegIsSure
parentSegStartBiIndex
parentSegEndBiIndex
isSure
```

Chan6 Rust 侧第一阶段至少需要：

```text
index
direction
start_fx_index
end_fx_index
start_bar_id
start_price
end_bar_id
end_price
confirmed
```

parent seg 相关字段预留到线段阶段。

### 2.6 SEG

路径：

```text
lib/core/models/seg.dart
```

语义字段：

```text
SegDirection.up
SegDirection.down
index
startBi
endBi
direction
isSure
reason
biList
prevIndex
nextIndex
```

Chan6 标准中：

```text
线段 = 1段
```

因此 Rust 中线段模型应明确为 1段，不能与更高 N段混淆。

### 2.7 RecursiveSEG

路径：

```text
lib/core/models/recursive_seg.dart
```

语义字段：

```text
layer
inputLayer
index
startParentIndex
endParentIndex
startRawIndex
endRawIndex
startPrice
endPrice
startTimeText
endTimeText
direction
isSure
```

Chan6 标准中：

```text
segseg = 2段，属于 chan.py 原生逻辑
3段及以上 = Chan6 基于 chan.py 的扩展逻辑
N 默认取最大可推导层级
```

因此 Rust 中应设计通用 `ChanNSegment` 模型，字段至少包括：

```text
n
input_n
index
start_parent_index
end_parent_index
start_bar_id
start_price
end_bar_id
end_price
direction
confirmed
```

### 2.8 RhythmLine / RhythmHit

路径：

```text
lib/core/models/rhythm.dart
```

`RhythmLine` 字段：

```text
id
level
sourceKind
sourceLabel
parentLevel
parentKey
calcMode
dir
displayLabel
labelLeft
labelRight
x1
y1
x2
y2
threshold
ratio
thresholdRatio
roundCurrent
roundRef
layer
```

`RhythmHit` 字段：

```text
id
lineId
level
sourceKind
rawIndex
time
price
threshold
dir
displayLabel
detail
```

Chan6 Rust 侧应改为：

```text
start_bar_id
start_price
end_bar_id
end_price
```

替代旧项目中的 `x1/x2/rawIndex` 作为业务锚点。

## 3. hichan 数据源注意事项

路径：

```text
lib/data/python_multi_level_chan_analysis_source.dart
```

旧项目通过 Python 分析源获取多级别缠论分析，并由 Dart 解析快照。

Chan6 不迁移该 Python 运行模式。Chan6 的原则是：

```text
Rust 后端计算
Flutter 前端渲染
```

因此该文件只作为旧系统数据契约和字段语义参考，不作为算法实现参考。

## 4. Chan6 下一步迁移顺序

云端开发应按以下顺序推进：

```text
1. Rust Chan 基础模型：ChanConfig / ChanBar / ChanMergedBar / ChanFx / ChanBi
2. KLine1m -> ChanBar 映射
3. 包含关系处理
4. FX 顶/底分型
5. BI 笔
6. query-chan-basic JSON 输出
7. 前端 ChanLayer 渲染 FX + BI
8. 线段/1段
9. segseg/2段
10. 3段及以上 N段
11. 节奏线
```

## 5. 强制约束

每次迁移 hichan 逻辑时必须确认：

```text
1. 是否仍然由 Rust 计算？
2. 是否把 rawIndex/index 转换为 bar_id？
3. 是否所有前端绘制对象都用 bar_id + price？
4. 是否没有把 Python 逻辑直接作为 Chan6 运行时依赖？
5. 是否保持线段=1段、segseg=2段、3段起扩展？
6. 是否为节奏线预留 Rust 输出契约？
```
