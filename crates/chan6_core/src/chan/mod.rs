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

pub mod config;
pub mod model;
pub mod standard;

pub use config::{ChanBiMode, ChanConfig, ChanFxMode, ChanIncludeMode, ChanSegmentN};
pub use model::{
    ChanBar, ChanBi, ChanDirection, ChanFx, ChanFxKind, ChanMergedBar, ChanRhythmHit,
    ChanRhythmLine, ChanSegment, CHAN_SEGMENT_N_EXTENSION_START, CHAN_SEGMENT_N_LINE,
    CHAN_SEGMENT_N_SEGSEG,
};
