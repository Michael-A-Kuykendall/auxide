use auxide::graph::{Graph, NodeType};
use auxide::plan::Plan;
use auxide::rt::Runtime;
use std::alloc::{GlobalAlloc, Layout};
use std::cell::RefCell;

thread_local! {
    static ALLOC_COUNT: RefCell<usize> = RefCell::new(0);
}

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.with(|c| *c.borrow_mut() += 1);
        unsafe { std::alloc::System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { std::alloc::System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static A: CountingAllocator = CountingAllocator;

#[test]
fn rt_alloc_invariant() {
    ALLOC_COUNT.with(|c| *c.borrow_mut() = 0);
    let mut out = vec![0.0; 64];
    ALLOC_COUNT.with(|c| *c.borrow_mut() = 0);
    let mut graph = Graph::new();
    let _node1 = graph.add_node(NodeType::Dummy);
    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);
    let after_new = ALLOC_COUNT.with(|c| *c.borrow());
    for _ in 0..10_000 {
        runtime.process_block(&mut out).unwrap();
    }
    let final_count = ALLOC_COUNT.with(|c| *c.borrow());
    assert_eq!(
        final_count, after_new,
        "RT process_block should not allocate"
    );
}
