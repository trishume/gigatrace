pub mod trace;
pub mod iforest;
pub mod index;

#[cfg(test)]
mod tests {
    use crate::trace::*;
    use crate::index::*;
    use crate::iforest::*;
    use fastrand::Rng;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn dummy_trace() {
        let mut pool = BlockPool::new();
        let mut track = Track::new();
        let rng = Rng::new();
        track.add_dummy_events(&mut pool, &rng, 300);

        let mut maxes = vec![];
        for i in &track.block_locs {
            maxes.push(LongestEvent::from_block(&pool.blocks[*i as usize]).0.unwrap().dur.to_u64());
        }
        println!("maxes: {:?}", maxes);

        let index = IForestIndex::<LongestEvent>::build(&track, &pool);
        let index_vals = index.vals.iter().map(|x| x.0.unwrap().dur.to_u64()).collect::<Vec<_>>();
        println!("index: {:?}", index_vals);

        // assert!(false);
        // TODO some test
    }

    #[test]
    fn block_count() {
        let mut pool = BlockPool::new();
        let mut track = Track::new();
        let rng = Rng::new();
        track.add_dummy_events(&mut pool, &rng, 325);

        let index = IForestIndex::<BlockCount>::build(&track, &pool);
        let index_vals = index.vals.iter().map(|x| x.0).collect::<Vec<_>>();
        println!("index: {:?}", index_vals);

        // assert!(false);
        // TODO some test
    }

    #[test]
    fn prop_test_range_query() {
        let mut pool = BlockPool::new();
        let mut track = Track::new();
        let rng = Rng::new();
        track.add_dummy_events(&mut pool, &rng, 325);

        let index = IForestIndex::<BlockCount>::build(&track, &pool);
        // let index_vals = index.vals.iter().map(|x| x.0).collect::<Vec<_>>();
        // println!("index: {:?}", index_vals);

        for _ in 0..100_000 {
            let start = rng.usize(..=track.block_locs.len());
            let end = rng.usize(start..=track.block_locs.len());
            let BlockCount(count) = index.range_query(start..end);
            assert_eq!(count,end-start, "failed for {}..{}", start, end);
        }
    }
}
