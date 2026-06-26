# Chan6 缠论实现标准

## 0. 总原则

Chan6 的缠论实现必须遵守以下规则：

1. 缠论计算逻辑使用 Rust 实现。
2. 不使用 Python 实现核心缠论算法。
3. Flutter 前端只负责显示、交互、图层管理，不参与缠论判定。
4. Rust 后端是分型、笔、线段、N段、中枢、买卖点、节奏线的唯一计算权威。
5. 所有图形对象必须使用 `bar_id + price` 锚定，不允许使用屏幕坐标作为业务坐标。
6. 缠论逻辑参考仓库 `git@github.com:cuixinyuan666/chan_replay_app.git` 的 `hichan` 分支。
7. 参考逻辑时必须先理解语义，再迁移到 Rust，不做机械翻译。
8. 每次修改缠论相关代码前，先检查本标准。
9. 线段即为 1段。
10. `segseg` 属于 2段，是 `chan.py` 原生逻辑范围。
11. 从 3段开始属于 Chan6 基于 `chan.py` 的扩展逻辑。
12. N段默认使用最大可推导值：持续递归升阶，直到不再满足下一层级的完整顶底结构。
13. 必须实现节奏线；节奏线由 Rust 后端计算，Flutter 只负责渲染。

## 1. 分层原则

### Rust 后端职责

Rust 负责：

- 原始 K线读取
- 包含关系处理
- 分型识别
- 笔识别
- 线段识别，也就是 1段识别
- `segseg` 识别，也就是 2段识别
- 3段及以上 N段递归升阶识别
- 中枢识别
- 买卖点识别
- 节奏线识别与输出
- 多级别推进
- 输出稳定 JSON 契约

### Flutter 前端职责

Flutter 负责：

- K线显示
- 缠论图层显示
- 节奏线图层显示
- 图层开关
- 鼠标/触摸交互
- 画线工具
- 命中检测
- 调用 Rust 查询接口
- 渲染 Rust 输出结果

Flutter 不允许重新判断：

- 分型是否成立
- 笔是否成立
- 线段是否成立
- `segseg` 是否成立
- N段是否成立
- 中枢是否成立
- 买卖点是否成立
- 节奏线是否成立

## 2. 数据锚定标准

所有缠论对象必须使用行情数据坐标：

```text
bar_id + price
```

允许：

```text
FX.bar_id
FX.price
BI.start_bar_id
BI.start_price
BI.end_bar_id
BI.end_price
SEG.start_bar_id
SEG.start_price
SEG.end_bar_id
SEG.end_price
SEGSEG.start_bar_id
SEGSEG.start_price
SEGSEG.end_bar_id
SEGSEG.end_price
NSEG.n
NSEG.start_bar_id
NSEG.start_price
NSEG.end_bar_id
NSEG.end_price
ZS.start_bar_id
ZS.end_bar_id
ZS.zg
ZS.zd
RHYTHM_LINE.start_bar_id
RHYTHM_LINE.start_price
RHYTHM_LINE.end_bar_id
RHYTHM_LINE.end_price
```

不允许把下面内容作为业务锚点：

```text
screen_x
screen_y
canvas_x
canvas_y
pixel_offset
```

屏幕坐标只能由前端临时计算，不允许写入后端缠论结果。

## 3. 段层级标准

Chan6 必须明确区分以下层级：

```text
线段 = 1段
segseg = 2段，属于 chan.py 原生逻辑
3段及以上 = Chan6 基于 chan.py 的扩展逻辑
```

### 3.1 默认 N 段行为

默认行为：

```text
N = max
```

含义：

```text
从线段开始递归升阶：
线段/1段 -> segseg/2段 -> 3段 -> 4段 -> ... -> N段

只要下一层级还能形成完整顶底结构，就继续向上推进。
直到下一层级不再满足完整顶底结构为止。
```

这里的“完整顶底结构”指该层级至少能识别出有效的顶结构和底结构，使该层级具备继续作为更高层级输入的基础。

### 3.2 显式指定 N

允许调用方显式指定 N，例如：

```text
n=1
n=2
n=3
```

但这只是查询或显示限制，不改变默认标准。

如果调用方没有显式指定 N，则必须使用最大可推导 N。

### 3.3 禁止行为

禁止：

```text
1. 默认只计算 1段
2. 默认只计算 2段
3. 写死最大 N
4. 因为前端暂时不显示就不计算更高 N段
5. 在 Flutter 中判断线段、segseg、N段顶底
```

## 4. 节奏线标准

节奏线必须纳入 Chan6 缠论实现范围。

### 4.1 计算权威

节奏线由 Rust 后端计算。

Flutter 只允许：

```text
1. 渲染 Rust 输出的节奏线
2. 控制节奏线图层显示/隐藏
3. 做鼠标命中和提示
```

Flutter 不允许：

```text
1. 自行判断节奏线成立
2. 自行生成节奏线业务对象
3. 使用屏幕坐标保存节奏线
```

### 4.2 参考来源

节奏线逻辑参考：

```text
git@github.com:cuixinyuan666/chan_replay_app.git
branch: hichan
```

迁移时必须先确认 hichan 中节奏线的输入、输出、过滤条件、可见性策略和命中语义，再用 Rust 实现。

### 4.3 输出要求

后续节奏线 JSON 输出必须能表达：

```text
id
kind
level
start_bar_id
start_price
end_bar_id
end_price
confirmed
source
visible
```

所有锚点仍然必须使用：

```text
bar_id + price
```

## 5. 开发顺序标准

缠论开发必须按以下顺序推进：

```text
1. 包含关系
2. 分型
3. 笔
4. 线段，也就是 1段
5. segseg，也就是 2段，chan.py 原生逻辑
6. 3段及以上 N段递归升阶，Chan6 扩展逻辑
7. 节奏线
8. 中枢
9. 买卖点
10. 多级别联动
11. 回测/选股接入
```

禁止跳过基础结构直接开发中枢或买卖点。

## 6. 第一阶段范围

第一阶段只实现：

```text
ChanConfig
ChanBar
ChanFx
ChanBi
包含关系处理
顶/底分型
笔
query-chan-basic
```

第一阶段不实现：

```text
线段/1段
segseg/2段
3段及以上 N段
节奏线
中枢
买卖点
背驰
多级别联动
```

第一阶段虽然不实现线段、N段和节奏线，但模型命名和 JSON 契约设计不能阻碍后续扩展。

## 7. JSON 输出标准

第一阶段 `query-chan-basic` 输出结构应稳定为：

```json
{
  "meta": {
    "query": "query-chan-basic",
    "schema_version": 1,
    "symbol": "002003",
    "level": "1m",
    "kline_count": 160,
    "fx_count": 12,
    "bi_count": 7,
    "include_mode": "standard",
    "fx_mode": "strict",
    "bi_mode": "normal"
  },
  "fx": [
    {
      "kind": "top",
      "bar_id": 1126530,
      "price": 9.86,
      "confirmed": true
    }
  ],
  "bi": [
    {
      "direction": "up",
      "start_bar_id": 1126510,
      "start_price": 9.72,
      "end_bar_id": 1126530,
      "end_price": 9.86,
      "confirmed": true
    }
  ]
}
```

后续扩展 JSON 必须预留：

```text
seg
segseg
nseg
rhythm_lines
zs
bs_points
```

## 8. hichan 参考标准

参考来源：

```text
git@github.com:cuixinyuan666/chan_replay_app.git
branch: hichan
```

迁移原则：

1. 先读取 hichan 当前实现。
2. 先整理算法语义。
3. 再用 Rust 重写。
4. 保留可测试的输入输出。
5. 不把 Flutter/Dart 逻辑搬到 Rust。
6. 不把 UI 状态当作缠论状态。
7. 如果 hichan 中存在临时 UI 适配逻辑，需要剥离。
8. `segseg` 作为 2段，按 chan.py 原生逻辑理解和迁移。
9. 3段及以上 N段作为 Chan6 扩展逻辑实现。
10. 节奏线按 hichan 语义迁移为 Rust 后端输出。

## 9. 每次修改缠论代码前的提醒

每次修改以下路径前：

```text
crates/chan6_core/src/chan/
crates/chan6_cli/
apps/chan6_app/lib/chart/layers/chan_layer.dart
apps/chan6_app/lib/chart/layers/rhythm_line_layer.dart
apps/chan6_app/lib/chart/data/chan_*.dart
```

必须确认：

```text
1. 是否仍然由 Rust 负责缠论计算？
2. 是否仍然用 bar_id + price 锚定？
3. 是否没有把屏幕坐标写入业务结果？
4. 是否没有在 Flutter 中重复实现缠论判断？
5. 是否保持 JSON 契约稳定？
6. 是否参考了 hichan 分支语义？
7. 是否遵守：线段=1段，segseg=2段，3段起为 Chan6 扩展？
8. 是否遵守：N 默认取最大可推导层级？
9. 是否遵守：节奏线由 Rust 后端输出，前端只渲染？
```
