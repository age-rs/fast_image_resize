use std::arch::wasm32::*;

use crate::convolution::optimisations::Normalizer32;
use crate::pixels::U16x3;
use crate::wasm32_utils;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_view: &impl ImageView<Pixel = U16x3>,
    dst_view: &mut impl ImageViewMut<Pixel = U16x3>,
    offset: u32,
    normalizer: &Normalizer32,
) {
    let dst_height = dst_view.height();

    let src_iter = src_view.iter_4_rows(offset, dst_height + offset);
    let dst_iter = dst_view.iter_4_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_four_rows(src_rows, dst_rows, normalizer);
        }
    }

    let yy = dst_height - dst_height % 4;
    let src_rows = src_view.iter_rows(yy + offset);
    let dst_rows = dst_view.iter_rows_mut(yy);
    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        unsafe {
            horiz_convolution_one_row(src_row, dst_row, normalizer);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - length of all rows in src_rows must be equal
/// - length of all rows in dst_rows must be equal
/// - coefficients_chunks.len() == dst_rows.0.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.0.len()
/// - precision <= MAX_COEFS_PRECISION
#[target_feature(enable = "simd128")]
unsafe fn horiz_convolution_four_rows(
    src_rows: [&[U16x3]; 4],
    dst_rows: [&mut [U16x3]; 4],
    normalizer: &Normalizer32,
) {
    const ZERO: v128 = i64x2(0, 0);
    let precision = normalizer.precision();
    let half_error = 1i64 << (precision - 1);
    let mut rg_buf = [0i64; 2];
    let mut bb_buf = [0i64; 2];

    /*
        |R    G    B   | |R    G    B   | |R    G   |
        |0001 0203 0405| |0607 0809 1011| |1213 1415|

        Shuffle to extract RG components of first pixel as i64:
        0, 1, -1, -1, -1, -1, -1, -1, 2, 3, -1, -1, -1, -1, -1, -1

        Shuffle to extract RG components of second pixel as i64:
        6, 7, -1, -1, -1, -1, -1, -1, 8, 9, -1, -1, -1, -1, -1, -1

        Shuffle to extract B components of two pixels as i64:
        4, 5, -1, -1, -1, -1, -1, -1, 10, 11, -1, -1, -1, -1, -1, -1

    */

    const RG0_SHUFFLE: v128 = i8x16(0, 1, -1, -1, -1, -1, -1, -1, 2, 3, -1, -1, -1, -1, -1, -1);
    const RG1_SHUFFLE: v128 = i8x16(6, 7, -1, -1, -1, -1, -1, -1, 8, 9, -1, -1, -1, -1, -1, -1);
    const BB_SHUFFLE: v128 = i8x16(4, 5, -1, -1, -1, -1, -1, -1, 10, 11, -1, -1, -1, -1, -1, -1);

    let width = src_rows[0].len();

    for (dst_x, coeffs_chunk) in normalizer.chunks().iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut rg_sum = [ZERO; 4];
        let mut bb_sum = [ZERO; 4];

        let mut coeffs = coeffs_chunk.values();
        let end_x = x + coeffs.len();

        if width - end_x >= 1 {
            let coeffs_by_2 = coeffs.chunks_exact(2);
            coeffs = coeffs_by_2.remainder();

            for k in coeffs_by_2 {
                let coeff0_i64x2 = i64x2_splat(k[0] as i64);
                let coeff1_i64x2 = i64x2_splat(k[1] as i64);
                let coeff_i64x2 = i64x2(k[0] as i64, k[1] as i64);

                for i in 0..4 {
                    let source = wasm32_utils::load_v128(src_rows[i], x);

                    let rg0_i64x2 = i8x16_swizzle(source, RG0_SHUFFLE);
                    rg_sum[i] = i64x2_add(
                        rg_sum[i],
                        wasm32_utils::i64x2_mul_lo(rg0_i64x2, coeff0_i64x2),
                    );

                    let rg1_i64x2 = i8x16_swizzle(source, RG1_SHUFFLE);
                    rg_sum[i] = i64x2_add(
                        rg_sum[i],
                        wasm32_utils::i64x2_mul_lo(rg1_i64x2, coeff1_i64x2),
                    );

                    let bb_i64x2 = i8x16_swizzle(source, BB_SHUFFLE);
                    bb_sum[i] =
                        i64x2_add(bb_sum[i], wasm32_utils::i64x2_mul_lo(bb_i64x2, coeff_i64x2));
                }
                x += 2;
            }
        }

        for &k in coeffs {
            let coeff_i64x2 = i64x2_splat(k as i64);

            for i in 0..4 {
                let &pixel = src_rows[i].get_unchecked(x);
                let rg_i64x2 = i64x2(pixel.0[0] as i64, pixel.0[1] as i64);
                rg_sum[i] = i64x2_add(rg_sum[i], wasm32_utils::i64x2_mul_lo(rg_i64x2, coeff_i64x2));
                let bb_i64x2 = i64x2(pixel.0[2] as i64, 0);
                bb_sum[i] = i64x2_add(bb_sum[i], wasm32_utils::i64x2_mul_lo(bb_i64x2, coeff_i64x2));
            }
            x += 1;
        }

        for i in 0..4 {
            v128_store(rg_buf.as_mut_ptr() as *mut v128, rg_sum[i]);
            v128_store(bb_buf.as_mut_ptr() as *mut v128, bb_sum[i]);
            let dst_pixel = dst_rows[i].get_unchecked_mut(dst_x);
            dst_pixel.0[0] = normalizer.clip(rg_buf[0] + half_error);
            dst_pixel.0[1] = normalizer.clip(rg_buf[1] + half_error);
            dst_pixel.0[2] = normalizer.clip(bb_buf[0] + bb_buf[1] + half_error);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coefficients_chunks.len() == dst_row.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[target_feature(enable = "simd128")]
unsafe fn horiz_convolution_one_row(
    src_row: &[U16x3],
    dst_row: &mut [U16x3],
    normalizer: &Normalizer32,
) {
    let precision = normalizer.precision();
    let rg_initial = i64x2_splat(1 << (precision - 1));
    let bb_initial = i64x2_splat(1 << (precision - 2));

    /*
        |R    G    B   | |R    G    B   | |R    G   |
        |0001 0203 0405| |0607 0809 1011| |1213 1415|

        Shuffle to extract RG components of first pixel as i64:
        0, 1, -1, -1, -1, -1, -1, -1, 2, 3, -1, -1, -1, -1, -1, -1

        Shuffle to extract RG components of second pixel as i64:
        6, 7, -1, -1, -1, -1, -1, -1, 8, 9, -1, -1, -1, -1, -1, -1

        Shuffle to extract B components of two pixels as i64:
        4, 5, -1, -1, -1, -1, -1, -1, 10, 11, -1, -1, -1, -1, -1, -1

    */

    const RG0_SHUFFLE: v128 = i8x16(0, 1, -1, -1, -1, -1, -1, -1, 2, 3, -1, -1, -1, -1, -1, -1);
    const RG1_SHUFFLE: v128 = i8x16(6, 7, -1, -1, -1, -1, -1, -1, 8, 9, -1, -1, -1, -1, -1, -1);
    const BB_SHUFFLE: v128 = i8x16(4, 5, -1, -1, -1, -1, -1, -1, 10, 11, -1, -1, -1, -1, -1, -1);
    let mut rg_buf = [0i64; 2];
    let mut bb_buf = [0i64; 2];

    let width = src_row.len();

    for (dst_x, coeffs_chunk) in normalizer.chunks().iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;

        let mut rg_sum = rg_initial;
        let mut bb_sum = bb_initial;

        let mut coeffs = coeffs_chunk.values();
        let end_x = x + coeffs.len();

        if width - end_x >= 1 {
            let coeffs_by_2 = coeffs.chunks_exact(2);
            coeffs = coeffs_by_2.remainder();

            for k in coeffs_by_2 {
                let coeff0_i64x2 = i64x2_splat(k[0] as i64);
                let coeff1_i64x2 = i64x2_splat(k[1] as i64);
                let coeff_i64x2 = i64x2(k[0] as i64, k[1] as i64);

                let source = wasm32_utils::load_v128(src_row, x);

                let rg0_i64x2 = i8x16_swizzle(source, RG0_SHUFFLE);
                rg_sum = i64x2_add(rg_sum, wasm32_utils::i64x2_mul_lo(rg0_i64x2, coeff0_i64x2));

                let rg1_i64x2 = i8x16_swizzle(source, RG1_SHUFFLE);
                rg_sum = i64x2_add(rg_sum, wasm32_utils::i64x2_mul_lo(rg1_i64x2, coeff1_i64x2));

                let bb_i64x2 = i8x16_swizzle(source, BB_SHUFFLE);
                bb_sum = i64x2_add(bb_sum, wasm32_utils::i64x2_mul_lo(bb_i64x2, coeff_i64x2));
                x += 2;
            }
        }

        for &k in coeffs {
            let coeff_i64x2 = i64x2_splat(k as i64);

            let &pixel = src_row.get_unchecked(x);
            let rg_i64x2 = i64x2(pixel.0[0] as i64, pixel.0[1] as i64);
            rg_sum = i64x2_add(rg_sum, wasm32_utils::i64x2_mul_lo(rg_i64x2, coeff_i64x2));
            let bb_i64x2 = i64x2(pixel.0[2] as i64, 0);
            bb_sum = i64x2_add(bb_sum, wasm32_utils::i64x2_mul_lo(bb_i64x2, coeff_i64x2));

            x += 1;
        }

        v128_store(rg_buf.as_mut_ptr() as *mut v128, rg_sum);
        v128_store(bb_buf.as_mut_ptr() as *mut v128, bb_sum);
        let dst_pixel = dst_row.get_unchecked_mut(dst_x);
        dst_pixel.0[0] = normalizer.clip(rg_buf[0]);
        dst_pixel.0[1] = normalizer.clip(rg_buf[1]);
        dst_pixel.0[2] = normalizer.clip(bb_buf[0] + bb_buf[1]);
    }
}
