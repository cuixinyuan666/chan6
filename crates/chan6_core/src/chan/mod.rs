//! Chan theory implementation modules.
//!
//! Standard behavior:
//! - Chan calculation is implemented in Rust.
//! - Flutter only renders Rust output.
//! - Business anchors are always `bar_id + price`, never screen coordinates.
//! - Algorithm semantics reference `chan_replay_app` branch `hichan`.
//! - Segment taxonomy: line segment = 1-segment, segseg = 2-segment,
//!   3-segment and above are Chan6 extensions based on chan.py semantics.
//! - The default N-segment behavior is max derivable N, not a fixed small N.
//! - Rhythm lines are computed by Rust and rendered by Flutter.

pub mod bi;
pub mod config;
pub mod engine;
pub mod fx;
pub mod include;
pub mod model;
pub mod segment;
pub mod zs;
pub mod standard;

pub use bi::{build_bis, build_bis_with_min_span, normalize_fxs_for_bi};
pub use config::{ChanBiMode, ChanConfig, ChanFxMode, ChanIncludeMode, ChanSegmentN};
pub use engine::{
    analyze_chan_basic, analyze_chan_basic_with_config, ChanBasicMeta, ChanBasicSnapshot,
    CHAN_BASIC_SCHEMA_VERSION,
};
pub use include::{has_include_relation, merge_included_bars};
pub use model::{
    ChanBar, ChanBi, ChanDirection, ChanFx, ChanFxKind, ChanMergedBar, ChanRhythmHit,
    ChanRhythmLine, ChanSegment, CHAN_SEGMENT_N_EXTENSION_START, CHAN_SEGMENT_N_LINE,
    CHAN_SEGMENT_N_SEGSEG,
};
pub use segment::{build_segments, build_segments_with_min_bi_count};
