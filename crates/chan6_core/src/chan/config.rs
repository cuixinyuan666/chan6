use serde::{Deserialize, Serialize};

/// Chan calculation configuration.
///
/// Defaults follow `docs/chan_implementation_standard.md`:
/// - calculation is Rust-authoritative;
/// - segment N defaults to the maximum derivable layer;
/// - line segment = 1, segseg = 2, N >= 3 is Chan6 extension.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChanConfig {
    pub include_mode: ChanIncludeMode,
    pub fx_mode: ChanFxMode,
    pub bi_mode: ChanBiMode,
    pub segment_n: ChanSegmentN,
    pub bsp: ChanBspConfig,
    pub enable_rhythm_lines: bool,
}

impl Default for ChanConfig {
    fn default() -> Self {
        Self {
            include_mode: ChanIncludeMode::Standard,
            fx_mode: ChanFxMode::Strict,
            bi_mode: ChanBiMode::Normal,
            segment_n: ChanSegmentN::MaxDerivable,
            bsp: ChanBspConfig::default(),
            enable_rhythm_lines: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChanBspConfig {
    pub enabled: bool,
    pub types: Vec<ChanBspType>,
    pub bsp2_follow_1: bool,
    pub bsp2s_follow_2: bool,
    pub max_bsp2_rate: f64,
    pub max_bsp2s_rate: f64,
    pub min_zs_cnt_for_t1p: usize,
    pub bsp3_follow_1: bool,
    pub bsp3_peak: bool,
    pub strict_bsp3: bool,
    pub bsp3a_max_zs_cnt: usize,
}

impl ChanBspConfig {
    pub fn is_type_enabled(&self, bsp_type: ChanBspType) -> bool {
        self.types.contains(&bsp_type)
    }
}

impl Default for ChanBspConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            types: vec![
                ChanBspType::T1,
                ChanBspType::T1p,
                ChanBspType::T2,
                ChanBspType::T2s,
                ChanBspType::T3a,
                ChanBspType::T3b,
            ],
            bsp2_follow_1: true,
            bsp2s_follow_2: true,
            max_bsp2_rate: 0.9999,
            max_bsp2s_rate: 0.9999,
            min_zs_cnt_for_t1p: 1,
            bsp3_follow_1: true,
            bsp3_peak: false,
            strict_bsp3: false,
            bsp3a_max_zs_cnt: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChanBspType {
    #[serde(rename = "1")]
    T1,
    #[serde(rename = "1p")]
    T1p,
    #[serde(rename = "2")]
    T2,
    #[serde(rename = "2s")]
    T2s,
    #[serde(rename = "3a")]
    T3a,
    #[serde(rename = "3b")]
    T3b,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChanIncludeMode {
    Standard,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChanFxMode {
    Strict,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChanBiMode {
    Normal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChanSegmentN {
    /// Continue segment promotion until the next layer no longer has a complete top/bottom structure.
    MaxDerivable,
    /// Restrict calculation or output to an explicit N.
    Explicit(u32),
}

impl ChanSegmentN {
    pub fn explicit_n(self) -> Option<u32> {
        match self {
            ChanSegmentN::MaxDerivable => None,
            ChanSegmentN::Explicit(n) => Some(n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ChanBspType, ChanConfig, ChanSegmentN};

    #[test]
    fn default_bsp_config_preserves_stage1_scope() {
        let config = ChanConfig::default();

        assert!(config.bsp.enabled);
        assert_eq!(
            config.bsp.types,
            vec![
                ChanBspType::T1,
                ChanBspType::T1p,
                ChanBspType::T2,
                ChanBspType::T2s,
                ChanBspType::T3a,
                ChanBspType::T3b,
            ]
        );
        assert!(config.bsp.bsp2_follow_1);
        assert!(config.bsp.bsp2s_follow_2);
        assert_eq!(config.bsp.max_bsp2_rate, 0.9999);
        assert_eq!(config.bsp.max_bsp2s_rate, 0.9999);
        assert_eq!(config.bsp.min_zs_cnt_for_t1p, 1);
    }

    #[test]
    fn default_segment_n_is_max_derivable() {
        let config = ChanConfig::default();
        assert_eq!(config.segment_n, ChanSegmentN::MaxDerivable);
        assert_eq!(config.segment_n.explicit_n(), None);
    }

    #[test]
    fn explicit_segment_n_is_supported_without_changing_default() {
        let segment_n = ChanSegmentN::Explicit(3);
        assert_eq!(segment_n.explicit_n(), Some(3));
        assert_eq!(ChanConfig::default().segment_n, ChanSegmentN::MaxDerivable);
    }
}
