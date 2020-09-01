use crate::trace::{TraceBlock, BlockPool, Track};
use crate::index::{Aggregate, TrackIndex};

pub struct IForestIndex<A: Aggregate> {
    pub indexed_count: usize,
    pub vals: Vec<A>,
}

impl<A: Aggregate> IForestIndex<A> {
    fn new() -> Self {
        IForestIndex { vals: vec![], indexed_count: 0 }
    }

    fn push(&mut self, block: &TraceBlock) {
        self.vals.push(A::from_block(block));

        // We want to index the first level every 2 nodes, 2nd level every 4 nodes...
        // This happens to correspond to the number of trailing ones in the index
        let levels_to_index = (!self.indexed_count).trailing_zeros();

        // Complete unfinished aggregation nodes which are now ready
        let len = self.vals.len();
        let mut cur = len-1; // The leaf we just pushed
        for level in 0..levels_to_index {
            let prev_higher_level = cur-(1 << level); // nodes at a level reach 2^level
            let combined = A::combine(&self.vals[prev_higher_level], &self.vals[cur]);
            self.vals[prev_higher_level] = combined;
            cur = prev_higher_level;
        }

        // Push new aggregation node going back one level further than we aggregated
        self.vals.push(self.vals[len-(1 << levels_to_index)].clone());

        self.indexed_count += 1;
    }
}

impl<A: Aggregate> TrackIndex<A> for IForestIndex<A> {
    fn build(track: &Track, pool: &BlockPool) -> IForestIndex<A> {
        let mut forest = IForestIndex::new();
        for i in &track.block_locs {
            forest.push(&pool.blocks[*i as usize]);
        }
        // TODO in parallel
        forest
    }
}
