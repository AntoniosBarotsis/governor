#![cfg(feature = "std")]

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use all_asserts::*;
use governor::{Quota, RateLimiter};
use nonzero_ext::*;
use serial_test::serial;
use std::hint::black_box;
use std::sync::Arc;
use std::thread;

const LEAK_TOLERANCE: usize = 2048;

struct LeakCheck {
    n_iter: usize,
    #[allow(dead_code)]
    profiler: dhat::Profiler,
}

impl LeakCheck {
    fn new(n_iter: usize) -> Self {
        LeakCheck {
            n_iter,
            profiler: dhat::Profiler::builder().testing().build(),
        }
    }
}

impl Drop for LeakCheck {
    fn drop(&mut self) {
        let stats = dhat::HeapStats::get();
        assert_le!(stats.curr_bytes, LEAK_TOLERANCE);
    }
}

#[test]
#[serial]
fn memleak_gcra() {
    let check = LeakCheck::new(500_000);
    {
        let bucket = RateLimiter::direct(Quota::per_second(nonzero!(1_000_000u32)));

        for _i in 0..check.n_iter {
            drop(black_box(bucket.check()));
        }
    }
}

#[test]
#[serial]
fn memleak_gcra_multi() {
    let check = LeakCheck::new(500_000);
    {
        let bucket = RateLimiter::direct(Quota::per_second(nonzero!(1_000_000u32)));

        for _i in 0..check.n_iter {
            drop(black_box(bucket.check_n(nonzero!(2u32))));
        }
    }
}

#[test]
#[serial]
fn memleak_gcra_threaded() {
    let check = LeakCheck::new(5_000);
    {
        let bucket = Arc::new(RateLimiter::direct(Quota::per_second(nonzero!(
            1_000_000u32
        ))));

        for _i in 0..check.n_iter {
            let bucket = Arc::clone(&bucket);
            thread::spawn(move || {
                assert_eq!(Ok(()), bucket.check());
            })
            .join()
            .unwrap();
        }
    }
}

#[test]
#[serial]
fn memleak_keyed() {
    let check = LeakCheck::new(500_000);
    {
        let bucket = RateLimiter::keyed(Quota::per_second(nonzero!(50u32)));

        for i in 0..check.n_iter {
            drop(black_box(bucket.check_key(&(i % 1000))));
        }
    }
}
