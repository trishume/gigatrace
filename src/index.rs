use crate::trace::{TraceBlock, TraceEvent, Track, BlockPool};

pub trait Aggregate: Clone {
    fn empty() -> Self;
    fn from_event(ev: &TraceEvent) -> Self;
    fn combine(&self, other: &Self) -> Self;

    fn from_block(block: &TraceBlock) -> Self {
        let mut c = Self::empty();
        for ev in block.events() {
            c = Self::combine(&c, &Self::from_event(ev));
        }
        c
    }
}

pub trait TrackIndex<A: Aggregate> {
    fn build(track: &Track, pool: &BlockPool) -> Self;
}

// === Concrete aggregations

#[derive(Clone)]
pub struct LongestEvent(pub Option<TraceEvent>);

impl Aggregate for LongestEvent {
    fn empty() -> Self {
        LongestEvent(None)
    }

    fn from_event(ev: &TraceEvent) -> Self {
        LongestEvent(Some(ev.clone()))
    }

    fn combine(&self, other: &Self) -> Self {
        LongestEvent([self.0, other.0].iter()
            .filter_map(|x| x.as_ref())
            .max_by_key(|ev| ev.dur.unpack())
            .map(|x| x.clone()))
    }

    fn from_block(block: &TraceBlock) -> Self {
        LongestEvent(block.events().iter().max_by_key(|ev| ev.dur.unpack()).map(|x| x.clone()))
    }
}

// #[derive(Clone)]
// pub struct LongestEventLoc {
//     dur: Ns,
//     index: usize,
// }

// impl Aggregate for LongestEventLoc {
//     fn empty() -> Self {
//         LongestEventLoc { dur: 0, index: usize::MAX }
//     }

//     fn from_event(ev: &TraceEvent) -> Self {
//         LongestEventLoc(Some(ev.clone()))
//     }

//     fn combine(&self, other: &Self) -> Self {
//         LongestEventLoc([self.0, other.0].iter()
//             .filter_map(|x| x.as_ref())
//             .max_by_key(|ev| ev.dur.unpack())
//             .map(|x| x.clone()))
//     }

//     fn from_block(block: &TraceBlock) -> Self {
//         LongestEventLoc(block.events().iter().max_by_key(|ev| ev.dur.unpack()).map(|x| x.clone()))
//     }
// }

/// For debugging
#[derive(Clone)]
pub struct EventCount(pub usize);

impl Aggregate for EventCount {
    fn empty() -> Self {
        EventCount(0)
    }

    fn from_event(_ev: &TraceEvent) -> Self {
        EventCount(1)
    }

    fn combine(&self, other: &Self) -> Self {
        EventCount(self.0 + other.0)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TsSum(pub u64);

impl Aggregate for TsSum {
    fn empty() -> Self {
        Self(0)
    }

    fn from_event(ev: &TraceEvent) -> Self {
        Self(ev.ts.unpack())
    }

    fn combine(&self, other: &Self) -> Self {
        Self(self.0 + other.0)
    }
}
