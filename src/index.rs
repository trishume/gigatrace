use crate::trace::{TraceBlock, TraceEvent, Track, BlockPool};

pub trait Aggregate: Default {
    fn from_block(block: &TraceBlock) -> Self;
    fn combine(&self, other: &Self) -> Self;
}

pub struct LongestEvent(pub Option<TraceEvent>);

impl LongestEvent {
}

impl Default for LongestEvent {
    fn default() -> Self {
        LongestEvent(None)
    }
}

impl Aggregate for LongestEvent {
    fn from_block(block: &TraceBlock) -> Self {
        LongestEvent(block.events().iter().max_by_key(|ev| ev.dur.to_u64()).map(|x| x.clone()))
    }

    fn combine(&self, other: &Self) -> Self {
        LongestEvent([self.0, other.0].iter()
            .filter_map(|x| x.as_ref())
            .max_by_key(|ev| ev.dur.to_u64())
            .map(|x| x.clone()))
    }
}

pub trait TrackIndex<A: Aggregate> {
    fn build(track: &Track, pool: &BlockPool) -> Self;
}
