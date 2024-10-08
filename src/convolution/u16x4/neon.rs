use std::arch::aarch64::*;

use crate::convolution::optimisations::{CoefficientsI32Chunk, Normalizer32};
use crate::neon_utils;
use crate::pixels::U16x4;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_view: &impl ImageView<Pixel = U16x4>,
    dst_view: &mut impl ImageViewMut<Pixel = U16x4>,
    offset: u32,
    normalizer: &Normalizer32,
) {
    let precision = normalizer.precision();

    macro_rules! call {
        ($imm8:expr) => {{
            horiz_convolution_p::<$imm8>(src_view, dst_view, offset, normalizer);
        }};
    }
    constify_64_imm8!(precision, call);
}

fn horiz_convolution_p<const PRECISION: i32>(
    src_view: &impl ImageView<Pixel = U16x4>,
    dst_view: &mut impl ImageViewMut<Pixel = U16x4>,
    offset: u32,
    normalizer: &Normalizer32,
) {
    let coefficients_chunks = normalizer.chunks();
    let dst_height = dst_view.height();

    let src_iter = src_view.iter_4_rows(offset, dst_height + offset);
    let dst_iter = dst_view.iter_4_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_four_rows::<PRECISION>(src_rows, dst_rows, coefficients_chunks);
        }
    }

    let yy = dst_height - dst_height % 4;
    let src_rows = src_view.iter_rows(yy + offset);
    let dst_rows = dst_view.iter_rows_mut(yy);
    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        unsafe {
            horiz_convolution_one_row::<PRECISION>(src_row, dst_row, coefficients_chunks);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - length of all rows in src_rows must be equal
/// - length of all rows in dst_rows must be equal
/// - coefficients_chunks.len() == dst_rows.0.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.0.len()
/// - precision <= MAX_COEFS_PRECISION
#[target_feature(enable = "neon")]
unsafe fn horiz_convolution_four_rows<const PRECISION: i32>(
    src_rows: [&[U16x4]; 4],
    dst_rows: [&mut [U16x4]; 4],
    coefficients_chunks: &[CoefficientsI32Chunk],
) {
    let initial = vdupq_n_s64(1i64 << (PRECISION - 1));
    let zero_u16x8 = vdupq_n_u16(0);
    let zero_u16x4 = vdup_n_u16(0);

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;

        let mut sss_a = [int64x2x2_t(initial, initial); 4];

        let mut coeffs = coeffs_chunk.values();

        let coeffs_by_8 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_8.remainder();
        for k in coeffs_by_8 {
            let coeffs_i32x4 = neon_utils::load_i32x4(k, 0);
            let coeff0 = vdup_laneq_s32::<0>(coeffs_i32x4);
            let coeff1 = vdup_laneq_s32::<1>(coeffs_i32x4);
            let coeff2 = vdup_laneq_s32::<2>(coeffs_i32x4);
            let coeff3 = vdup_laneq_s32::<3>(coeffs_i32x4);

            for i in 0..4 {
                let mut sss = sss_a[i];
                let source = neon_utils::load_u16x8(src_rows[i], x);

                let pix_i32 = vreinterpretq_s32_u16(vzip1q_u16(source, zero_u16x8));
                sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff0);
                sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff0);

                let pix_i32 = vreinterpretq_s32_u16(vzip2q_u16(source, zero_u16x8));
                sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff1);
                sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff1);

                let source = neon_utils::load_u16x8(src_rows[i], x + 2);

                let pix_i32 = vreinterpretq_s32_u16(vzip1q_u16(source, zero_u16x8));
                sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff2);
                sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff2);

                let pix_i32 = vreinterpretq_s32_u16(vzip2q_u16(source, zero_u16x8));
                sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff3);
                sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff3);

                sss_a[i] = sss;
            }

            x += 4;
        }

        let coeffs_by_4 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let coeffs_i32x2 = neon_utils::load_i32x2(k, 0);
            let coeff0 = vdup_lane_s32::<0>(coeffs_i32x2);
            let coeff1 = vdup_lane_s32::<1>(coeffs_i32x2);

            for i in 0..4 {
                let mut sss = sss_a[i];
                let source = neon_utils::load_u16x8(src_rows[i], x);

                let pix_i32 = vreinterpretq_s32_u16(vzip1q_u16(source, zero_u16x8));
                sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff0);
                sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff0);

                let pix_i32 = vreinterpretq_s32_u16(vzip2q_u16(source, zero_u16x8));
                sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff1);
                sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff1);

                sss_a[i] = sss;
            }
            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let coeff = vdup_n_s32(k);

            for i in 0..4 {
                let mut sss = sss_a[i];
                let source = vcombine_u16(neon_utils::load_u16x4(src_rows[i], x), zero_u16x4);

                let pix_i32 = vreinterpretq_s32_u16(vzip1q_u16(source, zero_u16x8));
                sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff);
                sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff);

                sss_a[i] = sss;
            }
        }

        sss_a[0].0 = vshrq_n_s64::<PRECISION>(sss_a[0].0);
        sss_a[0].1 = vshrq_n_s64::<PRECISION>(sss_a[0].1);
        sss_a[1].0 = vshrq_n_s64::<PRECISION>(sss_a[1].0);
        sss_a[1].1 = vshrq_n_s64::<PRECISION>(sss_a[1].1);
        sss_a[2].0 = vshrq_n_s64::<PRECISION>(sss_a[2].0);
        sss_a[2].1 = vshrq_n_s64::<PRECISION>(sss_a[2].1);
        sss_a[3].0 = vshrq_n_s64::<PRECISION>(sss_a[3].0);
        sss_a[3].1 = vshrq_n_s64::<PRECISION>(sss_a[3].1);

        for i in 0..4 {
            let sss = sss_a[i];
            let sss_i32x4 = vcombine_s32(vqmovn_s64(sss.0), vqmovn_s64(sss.1));
            let sss_u16x4 = vqmovun_s32(sss_i32x4);
            let dst_pix = dst_rows[i].get_unchecked_mut(dst_x);
            let ptr = dst_pix as *mut U16x4 as *mut u16;
            vst1_u16(ptr, sss_u16x4);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coefficients_chunks.len() == dst_row.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[target_feature(enable = "neon")]
unsafe fn horiz_convolution_one_row<const PRECISION: i32>(
    src_row: &[U16x4],
    dst_row: &mut [U16x4],
    coefficients_chunks: &[CoefficientsI32Chunk],
) {
    let initial = vdupq_n_s64(1i64 << (PRECISION - 1));
    let zero_u16x8 = vdupq_n_u16(0);
    let zero_u16x4 = vdup_n_u16(0);

    for (coeffs_chunk, dst_pix) in coefficients_chunks.iter().zip(dst_row) {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut sss = int64x2x2_t(initial, initial);
        let mut coeffs = coeffs_chunk.values();

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let coeffs_i32x4 = neon_utils::load_i32x4(k, 0);
            let source = neon_utils::load_u16x8(src_row, x);

            let coeff = vdup_laneq_s32::<0>(coeffs_i32x4);
            let pix_i32 = vreinterpretq_s32_u16(vzip1q_u16(source, zero_u16x8));
            sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff);
            sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff);

            let coeff = vdup_laneq_s32::<1>(coeffs_i32x4);
            let pix_i32 = vreinterpretq_s32_u16(vzip2q_u16(source, zero_u16x8));
            sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff);
            sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff);

            let source = neon_utils::load_u16x8(src_row, x + 2);

            let coeff = vdup_laneq_s32::<2>(coeffs_i32x4);
            let pix_i32 = vreinterpretq_s32_u16(vzip1q_u16(source, zero_u16x8));
            sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff);
            sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff);

            let coeff = vdup_laneq_s32::<3>(coeffs_i32x4);
            let pix_i32 = vreinterpretq_s32_u16(vzip2q_u16(source, zero_u16x8));
            sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff);
            sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff);

            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let coeffs_i32x2 = neon_utils::load_i32x2(k, 0);
            let source = neon_utils::load_u16x8(src_row, x);

            let coeff = vdup_lane_s32::<0>(coeffs_i32x2);
            let pix_i32 = vreinterpretq_s32_u16(vzip1q_u16(source, zero_u16x8));
            sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff);
            sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff);

            let coeff = vdup_lane_s32::<1>(coeffs_i32x2);
            let pix_i32 = vreinterpretq_s32_u16(vzip2q_u16(source, zero_u16x8));
            sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff);
            sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff);

            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let source = vcombine_u16(neon_utils::load_u16x4(src_row, x), zero_u16x4);

            let coeff = vdup_n_s32(k);
            let pix_i32 = vreinterpretq_s32_u16(vzip1q_u16(source, zero_u16x8));
            sss.0 = vmlal_s32(sss.0, vget_low_s32(pix_i32), coeff);
            sss.1 = vmlal_s32(sss.1, vget_high_s32(pix_i32), coeff);
        }

        sss.0 = vshrq_n_s64::<PRECISION>(sss.0);
        sss.1 = vshrq_n_s64::<PRECISION>(sss.1);

        let sss_i32x4 = vcombine_s32(vqmovn_s64(sss.0), vqmovn_s64(sss.1));
        let sss_u16x4 = vqmovun_s32(sss_i32x4);
        let ptr = dst_pix as *mut U16x4 as *mut u16;
        vst1_u16(ptr, sss_u16x4);
    }
}
