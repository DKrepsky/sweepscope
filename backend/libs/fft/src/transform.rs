use std::f64::consts::PI;

/// In-place iterative radix-2 decimation in time; the length is a power of two.
pub fn forward(re: &mut [f64], im: &mut [f64]) {
    let len = re.len();
    bit_reverse(re, im);
    let mut size = 2;
    while size <= len {
        butterflies(re, im, size);
        size *= 2;
    }
}

fn bit_reverse(re: &mut [f64], im: &mut [f64]) {
    let bits = re.len().trailing_zeros();
    if bits == 0 {
        return;
    }
    for i in 0..re.len() {
        let j = i.reverse_bits() >> (usize::BITS - bits);
        if j > i {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
}

fn butterflies(re: &mut [f64], im: &mut [f64], size: usize) {
    let half = size / 2;
    let step = -2.0 * PI / size as f64;
    for start in (0..re.len()).step_by(size) {
        for k in 0..half {
            let (sin, cos) = (step * k as f64).sin_cos();
            let (a, b) = (start + k, start + k + half);
            let tr = re[b] * cos - im[b] * sin;
            let ti = re[b] * sin + im[b] * cos;
            re[b] = re[a] - tr;
            im[b] = im[a] - ti;
            re[a] += tr;
            im[a] += ti;
        }
    }
}
