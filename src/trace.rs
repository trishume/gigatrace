use fastrand::Rng;

pub type Ns = u64;
#[derive(Copy, Clone)]
pub struct PackedNs([u8; 6]);

impl PackedNs {
    pub const fn new(ts: Ns) -> Self {
        let b = ts.to_le_bytes();
        // assert!(b[6] == 0 && b[7] == 0);
        PackedNs([b[0], b[1], b[2], b[3], b[4], b[5]])
    }

    #[inline]
    pub fn unpack(self) -> Ns {
        let b = self.0;
        u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], 0, 0])
    }
}

#[derive(Copy, Clone)]
pub struct TraceEvent {
    pub kind: u16,
    pub ts: PackedNs,
    pub dur: PackedNs,
}

pub const NULL_EVENT: TraceEvent = TraceEvent {
    kind: 0,
    ts: PackedNs::new(0),
    dur: PackedNs::new(0),
};

pub type BlockIndex = u32;

const EVENTS_PER_BLOCK: usize = 16;
pub struct TraceBlock {
    pub len: u16,
    events: [TraceEvent; EVENTS_PER_BLOCK],
}

impl TraceBlock {
    pub fn new() -> Self {
        Self {
            len: 0,
            events: [NULL_EVENT; EVENTS_PER_BLOCK],
        }
    }

    pub fn is_full(&self) -> bool {
        self.len as usize == EVENTS_PER_BLOCK
    }

    pub fn push(&mut self, ev: TraceEvent) {
        assert!(!self.is_full());
        self.events[self.len as usize] = ev;
        self.len += 1;
    }

    #[inline]
    pub fn events(&self) -> &[TraceEvent] {
        &self.events[..self.len as usize]
    }

    /// Returns 0 if block is empty, `Track` has a useful invariant that
    /// blocks are never empty.
    #[inline]
    pub fn start_time(&self) -> Ns {
        self.events[0].ts.unpack()
    }
}

pub struct BlockPool {
    pub blocks: Vec<TraceBlock>,
}

impl BlockPool {
    pub fn new() -> Self {
        BlockPool {
            blocks: vec![],
        }
    }

    pub fn alloc(&mut self) -> BlockIndex {
        let i = self.blocks.len();
        self.blocks.push(TraceBlock::new());
        i as BlockIndex
    }
}

pub struct Track {
    pub block_locs: Vec<BlockIndex>,
}

impl Track {
    pub fn new() -> Self {
        Self {
            block_locs: vec![]
        }
    }

    fn new_block(&mut self, pool: &mut BlockPool) -> BlockIndex {
        let i = pool.alloc();
        self.block_locs.push(i);
        i
    }

    pub fn push(&mut self, pool: &mut BlockPool, ev: TraceEvent) {
        let last = match self.block_locs.last() {
            None => self.new_block(pool),
            Some(&i) if pool.blocks[i as usize].is_full() => self.new_block(pool),
            Some(&i) => i
        };
        pool.blocks[last as usize].push(ev)
    }

    pub fn add_dummy_events(&mut self, pool: &mut BlockPool, rng: &Rng, n: usize) {
        let mut ts = 0;
        ts += rng.u64(..100_000);
        for _ in 0..n {
            ts += rng.u64(..10_000);
            let dur = rng.u64(..20_000);
            self.push(pool, TraceEvent {
                kind: rng.u16(4..15),
                ts: PackedNs::new(ts),
                dur: PackedNs::new(dur),
            });
            ts += dur;
        }
    }

    pub fn start_time(&self, pool: &BlockPool) -> Option<Ns> {
        self.block_locs.get(0).map(|i| pool.blocks[*i as usize].start_time())
    }

    pub fn end_time(&self, pool: &BlockPool) -> Option<Ns> {
        self.block_locs.last().and_then(|i| pool.blocks[*i as usize].events().last()).map(|x| x.ts.unpack())
    }
}
