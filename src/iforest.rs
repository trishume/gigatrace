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
        for level in 0..levels_to_index {
            let len = self.vals.len();
            // Left child is back a level-sized subtree, which are powers of two
            let combined = A::combine(&self.vals[len-(2 << level)], &self.vals[len-1]);
            self.vals.push(combined);
        }
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
