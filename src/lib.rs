pub mod dsl;
pub mod graph;
#[doc(hidden)]
pub mod harness;
#[doc(hidden)]
pub mod invariant_ppt;
pub mod plan;
pub mod rt;

/// Ingress particle: carries execution parameters.
#[derive(Debug, Clone)]
pub struct IngressParticle {
    pub block_size: usize,
    pub sample_rate: f32,
    pub channel_count: usize,
}

/// Egress particle: carries output results.
#[derive(Debug, Clone)]
pub struct EgressParticle {
    pub data: Vec<f32>,
    pub errors: Vec<String>, // Non-RT error channel
}

use std::alloc::{GlobalAlloc, Layout, System};

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        #[cfg(test)]
        {
            use crate::harness::ALLOC_COUNT;
            unsafe { ALLOC_COUNT += 1; }
        }
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[cfg(test)]
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;
