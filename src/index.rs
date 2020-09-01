use crate::trace::{TraceBlock, TraceEvent, Track, BlockPool};

pub trait Aggregate: Default + Clone {
    fn from_block(block: &TraceBlock) -> Self;
    fn combine(&self, other: &Self) -> Self;
}

pub trait TrackIndex<A: Aggregate> {
    fn build(track: &Track, pool: &BlockPool) -> Self;
}

// === Concrete aggregations

#[derive(Clone)]
pub struct LongestEvent(pub Option<TraceEvent>);

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

/// For debugging
#[derive(Clone)]
pub struct BlockCount(pub usize);

impl Default for BlockCount {
    fn default() -> Self {
        BlockCount(0)
    }
}

impl Aggregate for BlockCount {
    fn from_block(_block: &TraceBlock) -> Self {
        BlockCount(1)
    }

    fn combine(&self, other: &Self) -> Self {
        BlockCount(self.0 + other.0)
    }
}
