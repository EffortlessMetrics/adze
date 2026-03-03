#![no_main]

use adze_concurrency_env_contract_core::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS,
    RAYON_NUM_THREADS_ENV, TOKIO_WORKER_THREADS_ENV, parse_positive_usize_or_default,
};
use libfuzzer_sys::fuzz_target;

fn model_parse(value: Option<&str>, default: usize) -> usize {
    value
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|parsed| *parsed > 0)
        .unwrap_or(default)
}

fuzz_target!(|data: &[u8]| {
    let default_rayon = usize::from(data.first().copied().unwrap_or(0) % 64) + 1;
    let default_tokio = usize::from(data.get(1).copied().unwrap_or(0) % 64) + 1;
    let rest = if data.len() > 2 { &data[2..] } else { &[] };

    let split = rest
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(rest.len());
    let rayon_raw = String::from_utf8_lossy(&rest[..split]).to_string();
    let tokio_raw_slice = if split < rest.len() {
        &rest[split + 1..]
    } else {
        &[]
    };
    let tokio_raw = String::from_utf8_lossy(tokio_raw_slice).to_string();

    let parsed_rayon = parse_positive_usize_or_default(Some(&rayon_raw), default_rayon);
    let parsed_tokio = parse_positive_usize_or_default(Some(&tokio_raw), default_tokio);

    assert_eq!(parsed_rayon, model_parse(Some(&rayon_raw), default_rayon));
    assert_eq!(parsed_tokio, model_parse(Some(&tokio_raw), default_tokio));

    let caps = ConcurrencyCaps::from_lookup(|name| match name {
        RAYON_NUM_THREADS_ENV => Some(rayon_raw.clone()),
        TOKIO_WORKER_THREADS_ENV => Some(tokio_raw.clone()),
        _ => None,
    });

    assert_eq!(
        caps.rayon_threads,
        model_parse(Some(&rayon_raw), DEFAULT_RAYON_NUM_THREADS),
    );
    assert_eq!(
        caps.tokio_worker_threads,
        model_parse(Some(&tokio_raw), DEFAULT_TOKIO_WORKER_THREADS),
    );
});
