use adze_concurrency_init_rayon_core::init_rayon_global_once;
use proptest::prelude::*;

proptest! {
    #[test]
    fn repeated_init_calls_never_return_error(
        thread_counts in prop::collection::vec(0usize..256, 0..256),
    ) {
        for thread_count in thread_counts {
            prop_assert!(init_rayon_global_once(thread_count).is_ok());
        }
    }

    #[test]
    fn initialization_result_is_constant_after_first_call(
        first_threads in 0usize..256,
        second_threads in 0usize..256,
    ) {
        let first = init_rayon_global_once(first_threads);
        let second = init_rayon_global_once(second_threads);

        prop_assert_eq!(first, second);
    }
}
