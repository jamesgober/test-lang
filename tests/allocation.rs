//! How much memory `check` and `Diff::lines` use, measured with a counting
//! allocator.
//!
//! Each test binary has one global allocator, so the counters are shared by
//! every test here; the tests run one at a time under a lock.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use test_lang::{Change, Diff, Snapshot};

struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static SERIAL: Mutex<()> = Mutex::new(());

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        let live = LIVE.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
        PEAK.fetch_max(live, Ordering::Relaxed);
        // SAFETY: forwarded unchanged to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        // SAFETY: `ptr` came from `alloc` above with this layout.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// Run `f` and return (allocations made, peak bytes above the starting level).
fn measure<T>(f: impl FnOnce() -> T) -> (T, usize, usize) {
    let start_count = ALLOCATIONS.load(Ordering::Relaxed);
    let start_live = LIVE.load(Ordering::Relaxed);
    PEAK.store(start_live, Ordering::Relaxed);
    let out = f();
    let count = ALLOCATIONS.load(Ordering::Relaxed) - start_count;
    let peak = PEAK.load(Ordering::Relaxed).saturating_sub(start_live);
    (out, count, peak)
}

fn numbered(lines: usize, offset: usize) -> String {
    use std::fmt::Write;
    let mut text = String::new();
    for i in 0..lines {
        writeln!(text, "tok {}", i + offset).unwrap();
    }
    text
}

#[test]
fn test_matching_check_does_not_copy_every_line() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let text = numbered(10_000, 0);
    let snap = Snapshot::new(&text);
    let (result, allocations, _) = measure(|| snap.check(&text));
    assert!(result.is_ok());
    // Normalizing the expected text takes a handful of allocations; a copy of
    // each of the 10,000 lines would take 10,000 more.
    assert!(allocations < 100, "{allocations} allocations");
}

#[test]
fn test_large_diff_has_bounded_memory() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // Shifted by one, so the first and last lines differ and the whole text
    // is the changed region.
    let expected = numbered(8_000, 0);
    let actual = numbered(8_000, 1);
    let (diff, _, peak) = measure(|| Diff::lines(&expected, &actual));
    assert!(peak < 64 << 20, "peak {} MiB", peak >> 20);

    // The diff is still a correct edit script: replaying it gives both sides.
    let old: Vec<&str> = diff
        .changes()
        .filter(|(c, _)| *c != Change::Insert)
        .map(|(_, l)| l)
        .collect();
    let new: Vec<&str> = diff
        .changes()
        .filter(|(c, _)| *c != Change::Delete)
        .map(|(_, l)| l)
        .collect();
    assert_eq!(old.join("\n"), expected);
    assert_eq!(new.join("\n"), actual);
    assert!(!diff.is_empty());
}

#[test]
fn test_small_diff_is_still_minimal() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let expected = numbered(500, 0);
    let actual = numbered(500, 1);
    let diff = Diff::lines(&expected, &actual);
    let edits = diff.changes().filter(|(c, _)| *c != Change::Equal).count();
    // One line dropped at the top, one added at the bottom.
    assert_eq!(edits, 2);
}
