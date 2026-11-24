//! Sample-wise arithmetic for the math channel, over probe-scaled volts.
#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    InvertA,
}

/// `None` when `b` is needed and missing, or when the lengths differ. `InvertA` ignores `b`.
pub fn apply(operation: Operation, a: &[f64], b: Option<&[f64]>) -> Option<Vec<f64>> {
    match operation {
        Operation::InvertA => Some(a.iter().map(|x| -x).collect()),
        Operation::Add => combine(a, b?, |x, y| x + y),
        Operation::Subtract => combine(a, b?, |x, y| x - y),
        Operation::Multiply => combine(a, b?, |x, y| x * y),
    }
}

fn combine(a: &[f64], b: &[f64], f: impl Fn(f64, f64) -> f64) -> Option<Vec<f64>> {
    (a.len() == b.len()).then(|| a.iter().zip(b).map(|(&x, &y)| f(x, y)).collect())
}
