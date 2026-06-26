/// Chan6 Chan implementation standard.
///
/// This constant is intentionally kept inside Rust code so every Chan-related
/// development task can reference the required behavior before changing logic.
pub const CHAN_IMPLEMENTATION_STANDARD: &str = r#"
Chan6 缠论实现标准：

1. 缠论计算逻辑必须使用 Rust 实现。
2. 不使用 Python 实现核心缠论算法。
3. Flutter 前端只负责显示、交互、图层管理，不参与缠论判定。
4. Rust 后端是分型、笔、线段、N段、中枢、买卖点、节奏线、1.382、筹码分布、step 状态、BSP 特征、ML 打分的唯一计算权威。
5. 所有图形对象必须使用 bar_id + price 锚定。
6. 不允许使用 screen_x / screen_y / canvas_x / canvas_y 作为业务坐标。
7. 缠论逻辑参考 git@github.com:cuixinyuan666/chan_replay_app.git 的 hichan 分支。
8. 参考 hichan 时必须理解语义后用 Rust 重写，不做机械翻译。
9. 线段即为 1段。
10. segseg 属于 2段，是 chan.py 原生逻辑。
11. 从 3段开始属于 Chan6 基于 chan.py 的扩展逻辑。
12. N段默认使用最大可推导值，持续递归升阶，直到不再满足下一层级的完整顶底结构。
13. 节奏线必须实现，并由 Rust 后端计算；Flutter 只负责渲染。
14. 1.382 相关节奏/比例结构必须实现，并进入 BSP/ML 特征体系。
15. 筹码分布必须进入 BSP/ML 特征体系，且只能使用 step 当前 bar 及以前的累计筹码状态。
16. 第一阶段只实现 ChanConfig / ChanBar / ChanFx / ChanBi / 包含关系 / 分型 / 笔 / query-chan-basic。
17. step 模式必须作为后期逐 K 回放、回测、样本生成、BSP 特征提取和 ML 打分的基础。
18. BSP 特征必须与未来标签物理隔离，ML 打分只能依赖当时可见的特征，禁止未来函数。
19. 后续必须补充 ChanStepFrame / ChanBspPoint / ChanBspFeatureRow / ChanBspLabel / ChanBspMlScore。
"#;

pub fn print_chan_standard() {
    println!("{CHAN_IMPLEMENTATION_STANDARD}");
}

#[cfg(test)]
mod tests {
    use super::CHAN_IMPLEMENTATION_STANDARD;

    #[test]
    fn chan_standard_is_available_in_rust_code() {
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("Rust"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("bar_id + price"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("hichan"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("query-chan-basic"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("线段即为 1段"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("segseg"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("3段"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("最大可推导"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("节奏线"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("1.382"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("筹码分布"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("step"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("BSP 特征"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("ML 打分"));
        assert!(CHAN_IMPLEMENTATION_STANDARD.contains("未来函数"));
    }
}
