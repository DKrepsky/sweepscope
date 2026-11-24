use math_channel::{apply, Operation};

const A: [f64; 3] = [1.0, -2.0, 0.5];
const B: [f64; 3] = [0.5, 3.0, 4.0];

#[test]
fn given_a_and_b_when_added_then_each_sample_is_the_sum() {
    assert_eq!(apply(Operation::Add, &A, Some(&B)), Some(vec![1.5, 1.0, 4.5]));
}

#[test]
fn given_a_and_b_when_subtracted_then_each_sample_is_a_minus_b() {
    assert_eq!(apply(Operation::Subtract, &A, Some(&B)), Some(vec![0.5, -5.0, -3.5]));
}

#[test]
fn given_a_and_b_when_multiplied_then_each_sample_is_the_product() {
    assert_eq!(apply(Operation::Multiply, &A, Some(&B)), Some(vec![0.5, -6.0, 2.0]));
}

#[test]
fn given_invert_when_applied_then_b_is_ignored() {
    let expected = Some(vec![-1.0, 2.0, -0.5]);
    assert_eq!(apply(Operation::InvertA, &A, None), expected);
    assert_eq!(apply(Operation::InvertA, &A, Some(&[9.0])), expected);
}

#[test]
fn given_different_lengths_when_applied_then_the_result_is_none() {
    for op in [Operation::Add, Operation::Subtract, Operation::Multiply] {
        assert_eq!(apply(op, &A, Some(&B[..2])), None);
    }
}

#[test]
fn given_a_missing_b_when_a_binary_operation_is_applied_then_the_result_is_none() {
    assert_eq!(apply(Operation::Add, &A, None), None);
}

#[test]
fn given_empty_inputs_when_added_then_the_result_is_empty() {
    assert_eq!(apply(Operation::Add, &[], Some(&[])), Some(vec![]));
}
