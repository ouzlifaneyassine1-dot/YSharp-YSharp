#![allow(dead_code)]

pub fn conv2d(
    input: &[f64],
    kernel: &[f64],
    output: &mut [f64],
    in_h: usize,
    in_w: usize,
    kernel_h: usize,
    kernel_w: usize,
) {
    let out_h = in_h - kernel_h + 1;
    let out_w = in_w - kernel_w + 1;
    for oh in 0..out_h {
        for ow in 0..out_w {
            let mut sum = 0.0;
            for kh in 0..kernel_h {
                for kw in 0..kernel_w {
                    let ih = oh + kh;
                    let iw = ow + kw;
                    sum += input[ih * in_w + iw] * kernel[kh * kernel_w + kw];
                }
            }
            output[oh * out_w + ow] = sum;
        }
    }
}
