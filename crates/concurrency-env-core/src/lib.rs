//! Compatibility façade for `adze-concurrency-env-contract-core`.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_concurrency_env_contract_core::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reexported_defaults_are_accessible() {
        assert_eq!(DEFAULT_RAYON_NUM_THREADS, 4);
        assert_eq!(DEFAULT_TOKIO_WORKER_THREADS, 2);
    }

    #[test]
    fn reexported_caps_construct_via_lookup() {
        let caps = ConcurrencyCaps::from_lookup(|_| None);
        assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
        assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
    }

    #[test]
    fn reexported_parse_helper_works() {
        assert_eq!(parse_positive_usize_or_default(Some("5"), 1), 5);
        assert_eq!(parse_positive_usize_or_default(None, 7), 7);
        assert_eq!(parse_positive_usize_or_default(Some("0"), 3), 3);
    }

    #[test]
    fn reexported_caps_default_impl() {
        let caps = ConcurrencyCaps::default();
        assert_eq!(caps, ConcurrencyCaps::from_lookup(|_| None));
    }
}
