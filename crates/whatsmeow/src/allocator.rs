//! Memory tracking for FFI operations

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Custom allocator that tracks memory allocations for FFI operations.
#[derive(Default)]
pub struct TrackedAllocator {
    inner: System,
    allocation_count: AtomicUsize,
    deallocation_count: AtomicUsize,
    bytes_allocated: AtomicUsize,
    bytes_deallocated: AtomicUsize,
    peak_bytes: AtomicUsize,
    current_bytes: AtomicUsize,
}

impl TrackedAllocator {
    pub const fn new() -> Self {
        Self {
            inner: System,
            allocation_count: AtomicUsize::new(0),
            deallocation_count: AtomicUsize::new(0),
            bytes_allocated: AtomicUsize::new(0),
            bytes_deallocated: AtomicUsize::new(0),
            peak_bytes: AtomicUsize::new(0),
            current_bytes: AtomicUsize::new(0),
        }
    }

    /// Get total number of allocations
    pub fn allocation_count(&self) -> usize {
        self.allocation_count.load(Ordering::Relaxed)
    }

    /// Get total number of deallocations
    pub fn deallocation_count(&self) -> usize {
        self.deallocation_count.load(Ordering::Relaxed)
    }

    /// Get total bytes allocated (cumulative)
    pub fn total_bytes_allocated(&self) -> usize {
        self.bytes_allocated.load(Ordering::Relaxed)
    }

    /// Get total bytes deallocated (cumulative)
    pub fn total_bytes_deallocated(&self) -> usize {
        self.bytes_deallocated.load(Ordering::Relaxed)
    }

    /// Get current memory usage
    pub fn current_bytes(&self) -> usize {
        self.current_bytes.load(Ordering::Relaxed)
    }

    /// Get peak memory usage
    pub fn peak_bytes(&self) -> usize {
        self.peak_bytes.load(Ordering::Relaxed)
    }

    /// Get number of outstanding allocations (potential leaks)
    pub fn outstanding_allocations(&self) -> usize {
        self.allocation_count
            .load(Ordering::Relaxed)
            .saturating_sub(self.deallocation_count.load(Ordering::Relaxed))
    }

    /// Print memory statistics
    pub fn print_stats(&self) {
        println!("📊 Memory Statistics:");
        println!("   Allocations:   {}", self.allocation_count());
        println!("   Deallocations: {}", self.deallocation_count());
        println!("   Outstanding:   {}", self.outstanding_allocations());
        println!("   Current:       {} bytes", self.current_bytes());
        println!("   Peak:          {} bytes", self.peak_bytes());
        println!("   Total alloc:   {} bytes", self.total_bytes_allocated());
    }

    /// Trace an FFI operation with timing and memory tracking
    pub fn trace_operation<T, F: FnOnce() -> T>(&self, name: &str, f: F) -> T {
        let allocs_before = self.allocation_count();
        let bytes_before = self.current_bytes();
        let start = std::time::Instant::now();

        let result = f();

        let elapsed = start.elapsed();
        let allocs_after = self.allocation_count();
        let bytes_after = self.current_bytes();
        let allocs_delta = allocs_after.saturating_sub(allocs_before);
        let bytes_delta = bytes_after as i64 - bytes_before as i64;

        if elapsed.as_millis() > 10 || allocs_delta > 100 || bytes_delta.abs() > 10240 {
            tracing::debug!(
                operation = name,
                elapsed_ms = %elapsed.as_millis(),
                allocs = allocs_delta,
                bytes_delta = bytes_delta,
                "FFI operation (notable)"
            );
        } else {
            tracing::trace!(
                operation = name,
                elapsed_us = %elapsed.as_micros(),
                allocs = allocs_delta,
                "FFI operation"
            );
        }

        result
    }

    fn track_alloc(&self, size: usize) {
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        self.bytes_allocated.fetch_add(size, Ordering::Relaxed);
        let current = self.current_bytes.fetch_add(size, Ordering::Relaxed) + size;

        // Update peak if necessary
        let mut peak = self.peak_bytes.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_bytes.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }
    }

    fn track_dealloc(&self, size: usize) {
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
        self.bytes_deallocated.fetch_add(size, Ordering::Relaxed);
        self.current_bytes.fetch_sub(size, Ordering::Relaxed);
    }
}

unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.track_alloc(layout.size());
        unsafe { self.inner.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.track_dealloc(layout.size());
        unsafe { self.inner.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.track_alloc(layout.size());
        unsafe { self.inner.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // Track the size difference
        if new_size > layout.size() {
            self.track_alloc(new_size - layout.size());
        } else {
            self.track_dealloc(layout.size() - new_size);
        }
        unsafe { self.inner.realloc(ptr, layout, new_size) }
    }
}
