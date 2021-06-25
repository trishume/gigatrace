use crate::trace::{TraceBlock, BlockPool, Track};
use crate::index::{Aggregate, TrackIndex};
use std::ops::Range;

pub struct IForestIndex<A: Aggregate> {
    pub vals: Vec<A>,
}

//                #
// _______________|
// _______|_______|   #
// ___|___|___|___|___|
// 0|1|2|3|4|5|6|7|8|9|
impl<A: Aggregate> IForestIndex<A> {
    pub fn new() -> Self {
        IForestIndex { vals: vec![] }
    }

    pub fn push(&mut self, block: &TraceBlock) {
        self.vals.push(A::from_block(block));

        let len = self.vals.len();
        // We want to index the first level every 2 nodes, 2nd level every 4 nodes...
        // This happens to correspond to the number of trailing ones in the index
        let levels_to_index = len.trailing_ones()-1;

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

    /// See [havelessbemore's explanation] for more on why these bit tricks
    /// work. Thanks to him for the enhanced bit tricks for fewer branches.
    ///
    /// See the [old version] for a potentially easier to understand older
    /// version of this function that was a bit more complex and probably
    /// slower.
    ///
    /// [havelessbemore's explanation]: https://github.com/havelessbemore/dastal/blob/cd6a1d03872aa437f9272ce3fd42e2e2c006b2cc/src/segmentTree/inOrderSegmentTree.ts
    /// [old version]: https://github.com/trishume/gigatrace/blob/9e2fbb3c111529335f4f76a86ca788689dafd81c/src/iforest.rs
    pub fn range_query(&self, r: Range<usize>) -> A {
        /// offset past largest tree with left index x
        fn lsp(x: usize) -> usize {
            x & x.wrapping_neg() // leave the least significant bit
        }
        /// offset past largest tree up to x long
        fn msp(x: usize) -> usize {
            1usize.reverse_bits() >> x.leading_zeros() // leave the most significant bit
        }
        fn largest_prefix_inside_skip(min: usize, max: usize) -> usize {
            lsp(min|msp(max-min)) // = usize::min(lsp(min),msp(max-min))
        }
        fn agg_node(i: usize, offset: usize) -> usize {
            i + (offset >> 1) - 1 //
        }

        let mut ri = (r.start*2)..(r.end*2); // translate underlying to interior indices
        let len = self.vals.len();
        assert!(ri.start <= len && ri.end <= len, "range {:?} not inside 0..{}", r, len/2);

        let mut combined = A::empty();
        while ri.start < ri.end {
            let skip = largest_prefix_inside_skip(ri.start, ri.end);
            combined = A::combine(&combined, &self.vals[agg_node(ri.start, skip)]);
            ri.start += skip
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
