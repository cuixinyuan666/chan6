use serde::{Deserialize, Serialize};

/// Chan calculation configuration.
///
/// Defaults follow `docs/chan_implementation_standard.md`:
/// - calculation is Rust-authoritative;
/// - segment N defaults to the maximum derivable layer;
/// - line segment = 1, segseg = 2, N >= 3 is Chan6 extension.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChanConfig {
    pub include_mode: ChanIncludeMode,
    pub fx_mode: ChanFxMode,
    pub bi_mode: ChanBiMode,
    pub segment_n: ChanSegmentN,
    pub enable_rhythm_lines: bool,
}

impl Default for ChanConfig {
    fn default() -> Self {
        Self {
            include_mode: ChanIncludeMode::Standard,
            fx_mode: ChanFxMode::Strict,
            bi_mode: ChanBiMode::Normal,
            segment_n: ChanSegmentN::MaxDerivable,
            enable_rhythm_lines: true,
        }
    }
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
    use super::{ChanConfig, ChanSegmentN};

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
