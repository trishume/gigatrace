use crate::trace::{TraceBlock, BlockPool, Track};
use crate::index::{Aggregate, TrackIndex};
use std::ops::Range;

pub struct IForestIndex<A: Aggregate> {
    pub vals: Vec<A>,
}

impl<A: Aggregate> IForestIndex<A> {
    pub fn new() -> Self {
        IForestIndex { vals: vec![] }
    }

    pub fn push(&mut self, block: &TraceBlock) {
        self.vals.push(A::from_block(block));

        let len = self.vals.len();
        // We want to index the first level every 2 nodes, 2nd level every 4 nodes...
        // This happens to correspond to the number of trailing ones in the index
        let levels_to_index = (!(len/2)).trailing_zeros();

        // Complete unfinished aggregation nodes which are now ready
        let mut cur = len-1; // The leaf we just pushed
        for level in 0..levels_to_index {
            let prev_higher_level = cur-(1 << level); // nodes at a level reach 2^level
            let combined = A::combine(&self.vals[prev_higher_level], &self.vals[cur]);
            self.vals[prev_higher_level] = combined;
            cur = prev_higher_level;
        }

        // Push new aggregation node going back one level further than we aggregated
        self.vals.push(self.vals[len-(1 << levels_to_index)].clone());
    }

    pub fn range_query(&self, r: Range<usize>) -> A {
        fn left_child_at(node: usize, level: usize) -> bool {
            (node>>level)&1 == 0
        }
        fn skip(level: usize) -> usize {
            2<<level // lvl 0 skips self and agg node next to it, steps up by powers of 2
        }
        fn agg_node(node: usize, level: usize) -> usize {
            node+(1<<level)-1 // lvl 0 is us+0, lvl 1 is us+1, steps by power of 2
        }

        let mut ri = (r.start*2)..(r.end*2);
        let len = self.vals.len();
        assert!(ri.start <= len && ri.end <= len, "range {:?} not inside 0..{}", r, len/2);

        let mut combined: A = Default::default();
        while ri.start < ri.end {
            let mut up_level = 1;
            while left_child_at(ri.start, up_level) && ri.start+skip(up_level)<=ri.end {
                up_level += 1;
            }

            let level = up_level - 1;
            combined = A::combine(&combined, &self.vals[agg_node(ri.start, level)]);
            ri.start += skip(level);
        }

        combined
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
