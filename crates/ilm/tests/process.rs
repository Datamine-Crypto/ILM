use ilm_core::base::process::use_from;

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

unsafe extern "C" {
    fn mi_process_info(elapsed_msecs: *mut usize, user_msecs: *mut usize, system_msecs: *mut usize, current_rss: *mut usize, peak_rss: *mut usize, current_commit: *mut usize, peak_commit: *mut usize, page_faults: *mut usize);
}

#[test]
fn the_use_of_the_process_is_read_from_the_allocator() {
    let started = std::time::Instant::now();
    let mut turns: u64 = 0;
    while started.elapsed().as_millis() < 300 {
        turns = turns.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    }
    let kept: Vec<u64> = (0..4_000_000).map(|at| at ^ turns).collect();
    let used = use_from(mi_process_info).expect("the allocator reads the process");
    assert!(used.cpu_seconds > 0.1 && used.cpu_seconds < 1000.0, "{used:?}");
    assert!(used.peak_bytes >= kept.len() * 8, "{used:?} and {} bytes kept", kept.len() * 8);
}
