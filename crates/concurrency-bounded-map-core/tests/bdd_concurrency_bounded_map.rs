use adze_concurrency_bounded_map_core::{bounded_parallel_map, normalized_concurrency};

#[test]
fn given_zero_concurrency_when_normalizing_then_single_worker_is_used() {
    // Given / When
    let normalized = normalized_concurrency(0);

    // Then
    assert_eq!(normalized, 1);
}

#[test]
fn given_large_input_when_running_bounded_parallel_map_then_all_items_are_transformed() {
    // Given
    let input: Vec<i32> = (0..512).collect();

    // When
    let mut output = bounded_parallel_map(input.clone(), 4, |value| value * 3 + 1);

    // Then
    output.sort_unstable();
    let expected: Vec<i32> = input.into_iter().map(|value| value * 3 + 1).collect();
    assert_eq!(output, expected);
}

#[test]
fn given_empty_input_when_running_bounded_parallel_map_then_output_is_empty() {
    // Given
    let input: Vec<u8> = Vec::new();

    // When
    let output = bounded_parallel_map(input, 8, |value| value.saturating_add(1));

    // Then
    assert!(output.is_empty());
}
