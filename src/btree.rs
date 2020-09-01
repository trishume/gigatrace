#[derive(Copy, Clone)]
pub struct Ns([u8; 6]);

impl Ns {
    pub const fn new(ts: u64) -> Self {
        let b = ts.to_le_bytes();
        // assert!(b[6] == 0 && b[7] == 0);
        Ns([b[0], b[1], b[2], b[3], b[4], b[5]])
    }

    #[inline]
    pub fn to_u64(self) -> u64 {
        let b = self.0;
        u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], 0, 0])
    }
}

#[derive(Copy, Clone)]
pub struct TraceEvent {
    kind: u16,
    ts: Ns,
    dur: Ns,
}

const NULL_EVENT: TraceEvent = TraceEvent {
    kind: 0,
    ts: Ns::new(0),
    dur: Ns::new(0),
};

pub type NodeIndex = u32;

pub struct NodePool {
    root_index: NodeIndex,
    nodes: Vec<Node>,
}

type LevelIndex = u8;

pub struct Node {
    level: LevelIndex,
    allocated: u8,
    range: (Ns, Ns),
    body: NodeBody,
}

const EVENTS_PER_LEAF: usize = 16;
const SUBNODES_PER_NODE: usize = 16;
enum NodeBody {
    Leaf([TraceEvent; EVENTS_PER_LEAF]),
    Inner([NodeIndex; SUBNODES_PER_NODE]),
}

#[derive(Copy, Clone)]
enum NodeType {
    Leaf,
    Inner { level: LevelIndex },
}

#[derive(Debug)]
struct Overflow(NodeIndex);

impl NodePool {
    pub fn new() -> Self {
        let root = Node::new(NodeType::Leaf);
        NodePool {
            nodes: vec![root],
            root_index: 0,
        }
    }

    pub fn push(&mut self, ev: TraceEvent) {
        match self.push_into(self.root_index, ev) {
            Ok(()) => (),
            Err(Overflow(over_i)) => {
                // Allocate new root parent
                let old_level = self.nodes[self.root_index as usize].level;
                let new_root_i = self.alloc(NodeType::Inner {
                    level: old_level + 1,
                });
                let new_root = &mut self.nodes[new_root_i as usize];
                match new_root.body {
                    NodeBody::Leaf(_) => unreachable!(),
                    NodeBody::Inner(ref mut nodes) => {
                        nodes[0] = self.root_index;
                        nodes[1] = over_i;
                    }
                }
                new_root.allocated = 2;
                self.root_index = new_root_i;
            }
        }
    }

    fn alloc(&mut self, t: NodeType) -> NodeIndex {
        self.nodes.push(Node::new(t));
        (self.nodes.len() - 1) as u32
    }

    fn push_into(&mut self, node_i: NodeIndex, ev: TraceEvent) -> Result<(), Overflow> {
        let node = &self.nodes[node_i as usize];
        match node.body {
            NodeBody::Inner(nodes) => {
                let allocated = node.allocated;
                let level = node.level;
                assert!(allocated > 0);
                let last_child_i = nodes[(allocated-1) as usize];
                match self.push_into(last_child_i, ev) {
                    Ok(()) => Ok(()),
                    Err(Overflow(over_i)) if allocated as usize == SUBNODES_PER_NODE => {
                        // Allocate new sibling
                        let new_sibling_i = self.alloc(NodeType::Inner(level));

                    }
                    Err(Overflow(over_i)) => {
                        // Allocate new child
                        Ok(())
                    }
                }
            }
            NodeBody::Leaf(events) => {
                Ok(())
            }
        }
    }
}

impl Node {
    fn new(t: NodeType) -> Self {
        match t {
            NodeType::Leaf => Node {
                level: 0,
                allocated: 0,
                range: (Ns::new(0), Ns::new(0)),
                body: NodeBody::Leaf([NULL_EVENT; EVENTS_PER_LEAF]),
            },
            NodeType::Inner { level } => Node {
                level,
                allocated: 0,
                range: (Ns::new(0), Ns::new(0)),
                body: NodeBody::Inner([u32::MAX; SUBNODES_PER_NODE]),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
