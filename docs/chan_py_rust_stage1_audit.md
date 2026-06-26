# chan.py -> Rust 第一阶段审计记录

## 1. 本文件目的

本文件记录 Chan6 Rust 第一阶段模块与 `hichan/chan.py` 的对齐要求。

当前 Rust 模块：

```text
crates/chan6_core/src/chan/include.rs
crates/chan6_core/src/chan/fx.rs
crates/chan6_core/src/chan/bi.rs
crates/chan6_core/src/chan/engine.rs
```

在完成本文件所列审计前，只能称为第一版 Rust 骨架，不能称为严格复刻 `hichan/chan.py`。

## 2. 已定位的 hichan Python 后端入口

### 2.1 App 入口

```text
python/app_engine.py
```

作用：

```text
1. 将 backend 加入 sys.path
2. JSON 请求模式下导入 app.chanpy_engine.analyze_once / analyze_step
3. HTTP 模式下启动 app.main:app
```

### 2.2 chan.py 引擎包装层

```text
backend/app/chanpy_engine.py
```

关键点：

```text
1. _chanpy_path() 默认优先使用 python/chan.py
2. _prepare_chan() 加载 tools.chanpy_compare.chanpy_export
3. 通过 chanpy_export.import_chanpy() 导入 Chan.CChan / ChanConfig.CChanConfig
4. 使用 CChanConfig 构建配置
5. 使用 CChan 执行 chan.py 原生计算
6. _run_chanpy_export() 导出一次性结构
7. _run_chanpy_step_export() 通过 chan.step_load() 导出逐 K frames
```

### 2.3 chan.py 导出桥

```text
tools/chanpy_compare/chanpy_export.py
```

关键点：

```text
1. add_chanpy_path(path) 把 chan.py 根目录加入 sys.path
2. import_chanpy() 导入：
   - Chan.CChan
   - ChanConfig.CChanConfig
   - Common.CEnum.AUTYPE
   - Common.CEnum.DATA_SRC
   - Common.CEnum.KL_TYPE
3. make_cchan() 构造 CChan
4. export_fx(level) 导出 chan.py 原生分型
5. export_bi(level) 导出 chan.py 原生笔
6. export_seg(level) 导出 chan.py 原生线段
7. export_zs(level) 导出 chan.py 原生中枢
```

## 3. Rust 第一阶段必须对齐的 hichan 导出字段

### 3.1 merged_bars

hichan 导出位置：

```text
backend/app/chanpy_engine.py::_export_merged_bars(level)
```

导出字段：

```text
index
start_raw_index
end_raw_index
high_raw_index
low_raw_index
time
high_time
low_time
open
high
low
close
volume
```

Chan6 Rust 对应字段：

```text
index
start_bar_id
end_bar_id
high_bar_id
low_bar_id
open
high
low
close
volume
```

审计要求：

```text
1. Rust include.rs 必须对齐 chan.py 合并 K线结果
2. raw_index 必须映射为 bar_id
3. high_raw_index / low_raw_index 必须映射为 high_bar_id / low_bar_id
4. 不能只用经验规则判断包含关系
```

### 3.2 FX

hichan 导出位置：

```text
tools/chanpy_compare/chanpy_export.py::export_fx(level)
tools/chanpy_compare/chanpy_export.py::normalize_fx(i, item)
```

导出字段：

```text
index
raw_index
time
type
price
repr
```

Chan6 Rust 对应字段：

```text
index
bar_id
kind
price
confirmed
```

审计要求：

```text
1. Rust fx.rs 必须对齐 chan.py 原生 fx_list / fx_lst
2. raw_index 必须映射为 bar_id
3. top/bottom 的价格锚点必须与 chan.py export_fx 一致
4. 当前 Rust 严格三 K 分型规则只是骨架，需用 chan.py 输出校验
```

### 3.3 BI

hichan 导出位置：

```text
tools/chanpy_compare/chanpy_export.py::export_bi(level)
```

导出字段：

```text
index
start_raw_index
end_raw_index
start_time
end_time
start_price
end_price
direction
is_sure
repr
```

Chan6 Rust 对应字段：

```text
index
start_bar_id
end_bar_id
start_price
end_price
direction
confirmed
prev_index
next_index
```

审计要求：

```text
1. Rust bi.rs 必须对齐 chan.py 原生 bi_list / bi_lst
2. start_raw_index / end_raw_index 必须映射为 start_bar_id / end_bar_id
3. direction 必须以 chan.py 原生方向为准
4. is_sure 必须映射为 confirmed
5. 当前 Rust “同类分型去弱留强 + 异类成笔”只是骨架，需用 chan.py 输出校验
```

## 4. 下一步执行顺序

```text
1. 在 Chan6 中加入 chanpy 对照样例文档
2. 准备一组小型 K线输入样例
3. 用 hichan/chan.py 导出 merged_bars/fx/bi 金标准
4. 将金标准写入 Rust fixture
5. 修改 include.rs / fx.rs / bi.rs，直到 Rust 输出与 chan.py 金标准一致
6. 再继续 query-chan-basic CLI
```

## 5. 当前阻塞结论

在完成上述审计前，暂停继续扩展 CLI、前端、线段、N段、节奏线。
