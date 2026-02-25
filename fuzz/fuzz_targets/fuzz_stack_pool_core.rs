#![no_main]

use adze_stack_pool_core::StackPool;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let pool = StackPool::<u8>::new((data[0] as usize % 64) + 1);
    let mut stacks: Vec<Vec<u8>> = Vec::new();

    for &raw in &data[1..] {
        match raw % 5 {
            0 => {
                stacks.push(pool.acquire());
            }
            1 => {
                let capacity = usize::from(raw) % 256 + 1;
                stacks.push(pool.acquire_with_capacity(capacity));
            }
            2 => {
                if let Some(mut stack) = stacks.pop() {
                    for i in 0..(usize::from(raw) % 8) {
                        stack.push(i as u8);
                    }
                    pool.release(stack);
                }
            }
            3 => {
                if !stacks.is_empty() {
                    let index = (raw as usize) % stacks.len();
                    let mut copied = pool.clone_stack(&stacks[index]);
                    copied.push(raw);
                    stacks.push(copied);
                }
            }
            _ => {
                let stats = pool.stats();
                let _ = stats.total_allocations
                    + stats.reuse_count
                    + stats.pool_hits
                    + stats.pool_misses;
            }
        }
    }

    for stack in stacks {
        pool.release(stack);
    }
});
