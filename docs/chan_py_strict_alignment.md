# chan.py 严格对齐标准

本文件补充 `docs/chan_implementation_standard.md`。

## 1. 权威算法来源

Chan6 的 Rust 缠论实现目标是严格对齐：

```text
repository: cuixinyuan666/chan_replay_app
branch: hichan
engine: chan.py
```

`hichan` 中的 Flutter/Dart 模型层只作为字段语义参考，不作为算法权威。

## 2. Rust 实现原则

Rust 后端负责最终计算。迁移 `chan.py` 时必须做到：

```text
1. 先阅读 chan.py 原始逻辑
2. 再建立 chan.py -> Rust 的算法映射
3. 再建立可重复测试样例
4. 最后用 Rust 重写
```

禁止只凭经验近似实现。

## 3. 当前 Rust 状态

当前已经写入的 Rust 模块：

```text
crates/chan6_core/src/chan/include.rs
crates/chan6_core/src/chan/fx.rs
crates/chan6_core/src/chan/bi.rs
crates/chan6_core/src/chan/engine.rs
```

这些模块目前属于第一版 Rust 骨架和基础管线。

在完成 `chan.py` 审计前，不能把它们称为已经严格复刻 `hichan/chan.py`。

## 4. 后续必须执行的审计顺序

```text
1. 定位 hichan 分支中的 chan.py 和相关模块
2. 梳理包含关系算法
3. 梳理分型算法
4. 梳理笔算法
5. 梳理线段/segseg/N段算法
6. 梳理中枢和买卖点算法
7. 梳理节奏线和 1.382 算法
8. 对照当前 Rust include/fx/bi/engine
9. 不一致处以 chan.py 为准重写
10. 每一步补充 Rust 单元测试
```

## 5. 验收标准

后续每个 Rust 缠论模块必须能说明：

```text
1. 对应 chan.py 的哪个函数或类
2. 输入字段如何映射
3. 输出字段如何映射
4. 哪些规则已经严格对齐
5. 哪些规则还未迁移
6. 是否仍然使用 bar_id + price 作为业务锚点
```
