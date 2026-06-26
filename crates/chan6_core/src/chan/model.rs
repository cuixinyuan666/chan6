use serde::{Deserialize, Serialize};

use crate::model::KLine1m;

pub const CHAN_SEGMENT_N_LINE: u32 = 1;
pub const CHAN_SEGMENT_N_SEGSEG: u32 = 2;
pub const CHAN_SEGMENT_N_EXTENSION_START: u32 = 3;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChanDirection {
    Up,
    Down,
    Unknown,
}

impl ChanDirection {
    pub fn from_prices(start_price: f64, end_price: f64) -> Self {
        if end_price > start_price {
            ChanDirection::Up
        } else if end_price < start_price {
            ChanDirection::Down
        } else {
            ChanDirection::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChanFxKind {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChanBar {
    pub symbol: String,
    pub bar_id: i64,
    pub trading_day: i32,
    pub minute: i32,
    pub start_ts: i64,
    pub end_ts: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    pub trade_count: u32,
}

impl From<&KLine1m> for ChanBar {
    fn from(value: &KLine1m) -> Self {
        Self {
            symbol: value.symbol.clone(),
            bar_id: value.bar_id,
            trading_day: value.trading_day,
            minute: value.minute,
            start_ts: value.start_ts,
            end_ts: value.end_ts,
            open: value.open,
            high: value.high,
            low: value.low,
            close: value.close,
            volume: value.volume,
            amount: value.amount,
            trade_count: value.trade_count,
        }
    }
}

impl ChanBar {
    pub fn contains_price_range_of(&self, other: &Self) -> bool {
        self.high >= other.high && self.low <= other.low
    }

    pub fn is_contained_by(&self, other: &Self) -> bool {
        other.contains_price_range_of(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChanMergedBar {
    pub index: usize,
    pub symbol: String,
    pub start_bar_id: i64,
    pub end_bar_id: i64,
    /// Raw envelope high anchor. This matches hichan exported `high_raw_index`.
    pub high_bar_id: i64,
    /// Raw envelope low anchor. This matches hichan exported `low_raw_index`.
    pub low_bar_id: i64,
    /// Internal calculation high anchor after include-direction contraction.
    pub calc_high_bar_id: i64,
    /// Internal calculation low anchor after include-direction contraction.
    pub calc_low_bar_id: i64,
    pub trading_day: i32,
    pub minute: i32,
    pub start_ts: i64,
    pub end_ts: i64,
    pub open: f64,
    /// Raw envelope high for display/exported merged box.
    pub high: f64,
    /// Raw envelope low for display/exported merged box.
    pub low: f64,
    /// Internal calculation high used by FX/BI rules.
    pub calc_high: f64,
    /// Internal calculation low used by FX/BI rules.
    pub calc_low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    pub trade_count: u32,
}

impl ChanMergedBar {
    pub fn from_bar(index: usize, bar: &ChanBar) -> Self {
        Self {
            index,
            symbol: bar.symbol.clone(),
            start_bar_id: bar.bar_id,
            end_bar_id: bar.bar_id,
            high_bar_id: bar.bar_id,
            low_bar_id: bar.bar_id,
            calc_high_bar_id: bar.bar_id,
            calc_low_bar_id: bar.bar_id,
            trading_day: bar.trading_day,
            minute: bar.minute,
            start_ts: bar.start_ts,
            end_ts: bar.end_ts,
            open: bar.open,
            high: bar.high,
            low: bar.low,
            calc_high: bar.high,
            calc_low: bar.low,
            close: bar.close,
            volume: bar.volume,
            amount: bar.amount,
            trade_count: bar.trade_count,
        }
    }

    pub fn contains_bar_id(&self, bar_id: i64) -> bool {
        self.start_bar_id <= bar_id && bar_id <= self.end_bar_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChanFx {
    pub index: usize,
    pub kind: ChanFxKind,
    pub merged_index: usize,
    pub bar_id: i64,
    pub price: f64,
    pub confirmed: bool,
    pub left_merged_index: usize,
    pub center_merged_index: usize,
    pub right_merged_index: usize,
}

impl ChanFx {
    pub fn is_top(&self) -> bool {
        self.kind == ChanFxKind::Top
    }

    pub fn is_bottom(&self) -> bool {
        self.kind == ChanFxKind::Bottom
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChanBi {
    pub index: usize,
    pub direction: ChanDirection,
    pub start_fx_index: usize,
    pub end_fx_index: usize,
    pub start_bar_id: i64,
    pub start_price: f64,
    pub end_bar_id: i64,
    pub end_price: f64,
    pub confirmed: bool,
    pub prev_index: Option<usize>,
    pub next_index: Option<usize>,
    pub parent_segment_index: Option<usize>,
}

impl ChanBi {
    pub fn new(
        index: usize,
        start_fx_index: usize,
        end_fx_index: usize,
        start_bar_id: i64,
        start_price: f64,
        end_bar_id: i64,
        end_price: f64,
    ) -> Self {
        Self {
            index,
            direction: ChanDirection::from_prices(start_price, end_price),
            start_fx_index,
            end_fx_index,
            start_bar_id,
            start_price,
            end_bar_id,
            end_price,
            confirmed: true,
            prev_index: None,
            next_index: None,
            parent_segment_index: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChanSegment {
    /// Segment layer. 1 = line segment, 2 = segseg, 3+ = Chan6 extension.
    pub n: u32,
    pub input_n: Option<u32>,
    pub index: usize,
    pub direction: ChanDirection,
    pub start_parent_index: Option<usize>,
    pub end_parent_index: Option<usize>,
    pub start_bar_id: i64,
    pub start_price: f64,
    pub end_bar_id: i64,
    pub end_price: f64,
    pub confirmed: bool,
    pub reason: String,
}

impl ChanSegment {
    pub fn is_line_segment(&self) -> bool {
        self.n == CHAN_SEGMENT_N_LINE
    }

    pub fn is_segseg(&self) -> bool {
        self.n == CHAN_SEGMENT_N_SEGSEG
    }

    pub fn is_extension_n_segment(&self) -> bool {
        self.n >= CHAN_SEGMENT_N_EXTENSION_START
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChanRhythmLine {
    pub id: String,
    pub level: String,
    pub source_kind: String,
    pub source_label: String,
    pub parent_level: String,
    pub parent_key: String,
    pub calc_mode: String,
    pub direction: ChanDirection,
    pub display_label: String,
    pub label_left: String,
    pub label_right: String,
    pub start_bar_id: i64,
    pub start_price: f64,
    pub end_bar_id: i64,
    pub end_price: f64,
    pub threshold: f64,
    pub ratio: f64,
    pub threshold_ratio: f64,
    pub round_current: i32,
    pub round_ref: i32,
    pub layer: u32,
    pub confirmed: bool,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChanRhythmHit {
    pub id: String,
    pub line_id: String,
    pub level: String,
    pub source_kind: String,
    pub bar_id: i64,
    pub price: f64,
    pub threshold: f64,
    pub direction: ChanDirection,
    pub display_label: String,
    pub detail: String,
}

#[cfg(test)]
mod tests {
    use super::{
        ChanBar, ChanBi, ChanDirection, ChanSegment, CHAN_SEGMENT_N_EXTENSION_START,
        CHAN_SEGMENT_N_LINE, CHAN_SEGMENT_N_SEGSEG,
    };
    use crate::model::KLine1m;

    #[test]
    fn kline_maps_to_chan_bar_using_bar_id_anchor() {
        let kline = KLine1m {
            symbol: "002003".to_string(),
            bar_id: 1126619,
            trading_day: 20260511,
            minute: 1121,
            start_ts: 1,
            end_ts: 2,
            open: 9.82,
            high: 9.83,
            low: 9.81,
            close: 9.82,
            volume: 10.0,
            amount: 98.2,
            trade_count: 3,
        };

        let chan_bar = ChanBar::from(&kline);
        assert_eq!(chan_bar.symbol, "002003");
        assert_eq!(chan_bar.bar_id, 1126619);
        assert_eq!(chan_bar.trading_day, 20260511);
        assert_eq!(chan_bar.minute, 1121);
    }

    #[test]
    fn bi_direction_is_derived_from_price_endpoints() {
        let up = ChanBi::new(0, 0, 1, 10, 9.0, 20, 10.0);
        let down = ChanBi::new(1, 1, 2, 20, 10.0, 30, 9.0);
        let flat = ChanBi::new(2, 2, 3, 30, 9.0, 40, 9.0);

        assert_eq!(up.direction, ChanDirection::Up);
        assert_eq!(down.direction, ChanDirection::Down);
        assert_eq!(flat.direction, ChanDirection::Unknown);
    }

    #[test]
    fn segment_taxonomy_matches_chan_standard() {
        let segment = sample_segment(CHAN_SEGMENT_N_LINE);
        let segseg = sample_segment(CHAN_SEGMENT_N_SEGSEG);
        let extension = sample_segment(CHAN_SEGMENT_N_EXTENSION_START);

        assert!(segment.is_line_segment());
        assert!(!segment.is_segseg());
        assert!(segseg.is_segseg());
        assert!(extension.is_extension_n_segment());
    }

    fn sample_segment(n: u32) -> ChanSegment {
        ChanSegment {
            n,
            input_n: if n > 1 { Some(n - 1) } else { None },
            index: 0,
            direction: ChanDirection::Up,
            start_parent_index: Some(0),
            end_parent_index: Some(1),
            start_bar_id: 1,
            start_price: 9.0,
            end_bar_id: 10,
            end_price: 10.0,
            confirmed: true,
            reason: "test".to_string(),
        }
    }
}
