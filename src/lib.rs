pub mod iforest;
pub mod index;
pub mod trace;

use iforest::IForestIndex;
use index::Aggregate;
use trace::{BlockPool, Ns, BlockIndex};
use std::ops::Range;
use std::mem;

pub fn aggregate_by_steps<A: Aggregate>(
    pool: &BlockPool,
    block_locs: &[BlockIndex],
    index: &IForestIndex<A>,
    time_span: Range<Ns>,
    time_step: u64,
) -> Vec<A> {
    let mut out = vec![];

    let mut block_i = 0;
    let mut target_time = time_span.start;
    let mut combined = A::empty();
    'outer: loop {
        if block_i >= block_locs.len() {
            break;
        }

        // == Skip to last block with a start_time before target_time
        let bsearch_res = block_locs[block_i..]
            .binary_search_by_key(&target_time, |i| pool.blocks[*i as usize].start_time())
            .unwrap_or_else(|i| i);
        if bsearch_res > 1 {
            let skip = bsearch_res - 1;
            combined = A::combine(&combined, &index.range_query(block_i..(block_i+skip)));
            block_i += skip;
        }
        // == aggregate range using the index

        let block = &pool.blocks[block_locs[block_i] as usize];
        for ev in block.events() {
            let ev_ts = ev.ts.unpack();
            while ev_ts >= target_time {
                // TODO add trait bool fn to allow skipping adding empty stuff to lists
                out.push(mem::replace(&mut combined, A::empty()));
                if target_time >= time_span.end {
                    break 'outer;
                }
                target_time = target_time + time_step;
            }
            combined = A::combine(&combined, &A::from_event(ev));
        }

        block_i += 1;
    }

    out
}

pub fn aggregate_by_steps_unindexed<A: Aggregate>(
    pool: &BlockPool,
    block_locs: &[BlockIndex],
    time_span: Range<Ns>,
    time_step: u64,
) -> Vec<A> {
    let mut out = vec![];

    let mut target_time = time_span.start;
    let mut combined = A::empty();
    'outer: for block_i in block_locs {
        let block = &pool.blocks[*block_i as usize];
        for ev in block.events() {
            let ev_ts = ev.ts.unpack();
            while ev_ts >= target_time {
                // TODO add trait bool fn to allow skipping adding empty stuff to lists
                out.push(mem::replace(&mut combined, A::empty()));
                if target_time >= time_span.end {
                    break 'outer;
                }
                target_time = target_time + time_step;
            }
            combined = A::combine(&combined, &A::from_event(ev));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::iforest::*;
    use crate::index::*;
    use crate::trace::*;
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
            maxes.push(
                LongestEvent::from_block(&pool.blocks[*i as usize])
                    .0
                    .unwrap()
                    .dur
                    .unpack(),
            );
        }
        println!("maxes: {:?}", maxes);

        let index = IForestIndex::<LongestEvent>::build(&track, &pool);
        let index_vals = index
            .vals
            .iter()
            .map(|x| x.0.unwrap().dur.unpack())
            .collect::<Vec<_>>();
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

        let index = IForestIndex::<EventCount>::build(&track, &pool);
        let index_vals = index.vals.iter().map(|x| x.0).collect::<Vec<_>>();
        println!("index: {:?}", index_vals);

        // assert!(false);
        // TODO some test
    }

    #[test]
    fn aggregate_by_steps_unindexed() {
        let mut pool = BlockPool::new();
        let mut track = Track::new();
        let ev_ts = &[10, 15, 20, 100, 101, 150, 170];
        for t in ev_ts {
            track.push(&mut pool, TraceEvent {
                kind: 0,
                ts: PackedNs::new(*t),
                dur: PackedNs::new(0),
            });
        }

        let span = 13..150;
        let res = crate::aggregate_by_steps_unindexed::<TsSum>(&pool, &track.block_locs, span, 10);
        let res_ts = res.iter().map(|x| x.0).collect::<Vec<_>>();
        assert_eq!(&res_ts[..], &[10, 35, 0, 0, 0, 0, 0, 0, 0, 201, 0, 0, 0, 0, 150]);
    }

    #[test]
    fn prop_test_range_query() {
        let mut pool = BlockPool::new();
        let mut track = Track::new();
        let rng = Rng::new();
        track.add_dummy_events(&mut pool, &rng, 325);

        let index = IForestIndex::<EventCount>::build(&track, &pool);
        // let index_vals = index.vals.iter().map(|x| x.0).collect::<Vec<_>>();
        // println!("index: {:?}", index_vals);

        for _ in 0..100_000 {
            let start = rng.usize(..=track.block_locs.len());
            let end = rng.usize(start..=track.block_locs.len());
            let EventCount(count) = index.range_query(start..end);
            let correct: usize = track.block_locs[start..end].iter()
                .map(|i| pool.blocks[*i as usize].len as usize).sum();
            assert_eq!(count, correct, "failed for {}..{}", start, end);
        }
    }


    #[test]
    fn prop_test_aggregate_by_steps() {
        let mut pool = BlockPool::new();
        let mut track = Track::new();
        let rng = Rng::new();
        track.add_dummy_events(&mut pool, &rng, 325);

        let index = IForestIndex::<TsSum>::build(&track, &pool);
        // let index_vals = index.vals.iter().map(|x| x.0).collect::<Vec<_>>();
        // println!("index: {:?}", index_vals);

        let time_bounds = 0..=(track.end_time(&pool).unwrap()+100_000);
        for _ in 0..100_000 {
            let t1 = rng.u64(time_bounds.clone());
            let t2 = rng.u64(time_bounds.clone());
            let t_range = if t2 > t1 { t1..t2 } else { t2..t1 };
            let range_size = t_range.end - t_range.start;
            let step = (range_size / rng.u64(1..10)) + rng.u64(0..100);
            let res1 = crate::aggregate_by_steps::<TsSum>(&pool, &track.block_locs, &index, t_range.clone(), step);
            let res2 = crate::aggregate_by_steps_unindexed::<TsSum>(&pool, &track.block_locs, t_range.clone(), step);
            assert_eq!(res1, res2, "failed for {:?} - {}", t_range, step);
        }
    }
}
