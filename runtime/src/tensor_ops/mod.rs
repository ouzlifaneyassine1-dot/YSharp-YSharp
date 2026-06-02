#![allow(dead_code)]

pub mod matmul;
pub mod conv;

pub fn gemm(a: &[f64], b: &[f64], c: &mut [f64], m: usize, n: usize, k: usize) {
    matmul::matmul(a, b, c, m, n, k);
}
