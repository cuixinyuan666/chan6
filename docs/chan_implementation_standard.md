# Chan6 缠论实现标准

## 0. 总原则

Chan6 的缠论实现必须遵守以下规则：

1. 缠论计算逻辑使用 Rust 实现。
2. 不使用 Python 实现核心缠论算法。
3. Flutter 前端只负责显示、交互、图层管理，不参与缠论判定。
4. Rust 后端是分型、笔、线段、中枢、买卖点的唯一计算权威。
5. 所有图形对象必须使用 `bar_id + price` 锚定，不允许使用屏幕坐标作为业务坐标。
6. 缠论逻辑参考仓库 `git@github.com:cuixinyuan666/chan_replay_app.git` 的 `hichan` 分支。
7. 参考逻辑时必须先理解语义，再迁移到 Rust，不做机械翻译。
8. 每次修改缠论相关代码前，先检查本标准。

## 1. 分层原则

### Rust 后端职责

Rust 负责：

- 原始 K线读取
- 包含关系处理
- 分型识别
- 笔识别
- 线段识别
- 中枢识别
- 买卖点识别
- 多级别推进
- 输出稳定 JSON 契约

### Flutter 前端职责

Flutter 负责：

- K线显示
- 缠论图层显示
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
- 中枢是否成立
- 买卖点是否成立

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
ZS.start_bar_id
ZS.end_bar_id
ZS.zg
ZS.zd
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

## 3. 开发顺序标准

缠论开发必须按以下顺序推进：

```text
1. 包含关系
2. 分型
3. 笔
4. 线段
5. 中枢
6. 买卖点
7. 多级别联动
8. 回测/选股接入
```

禁止跳过基础结构直接开发中枢或买卖点。

## 4. 第一阶段范围

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
线段
中枢
买卖点
背驰
多级别联动
```

## 5. JSON 输出标准

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

## 6. hichan 参考标准

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

## 7. 每次修改缠论代码前的提醒

每次修改以下路径前：

```text
crates/chan6_core/src/chan/
crates/chan6_cli/
apps/chan6_app/lib/chart/layers/chan_layer.dart
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
```
