use crate::alpha::common::{div_and_clip, mul_div_255, RECIP_ALPHA};
use crate::pixels::U8x2;
use crate::{ImageView, ImageViewMut};

pub(crate) fn multiply_alpha(
    src_view: &impl ImageView<Pixel = U8x2>,
    dst_view: &mut impl ImageViewMut<Pixel = U8x2>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row(src_row, dst_row);
    }
}

pub(crate) fn multiply_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = U8x2>) {
    for row in image_view.iter_rows_mut(0) {
        multiply_alpha_row_inplace(row);
    }
}

#[inline(always)]
pub(crate) fn multiply_alpha_row(src_row: &[U8x2], dst_row: &mut [U8x2]) {
    for (src_pixel, dst_pixel) in src_row.iter().zip(dst_row) {
        let components: [u8; 2] = src_pixel.0;
        let alpha = components[1];
        dst_pixel.0 = [mul_div_255(components[0], alpha), alpha];
    }
}

#[inline(always)]
pub(crate) fn multiply_alpha_row_inplace(row: &mut [U8x2]) {
    for pixel in row {
        let components: [u8; 2] = pixel.0;
        let alpha = components[1];
        pixel.0 = [mul_div_255(components[0], alpha), alpha];
    }
}

// Divide

#[inline]
pub(crate) fn divide_alpha(
    src_view: &impl ImageView<Pixel = U8x2>,
    dst_view: &mut impl ImageViewMut<Pixel = U8x2>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[inline]
pub(crate) fn divide_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = U8x2>) {
    for dst_row in image_view.iter_rows_mut(0) {
        let src_row = unsafe { std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len()) };
        divide_alpha_row(src_row, dst_row);
    }
}

#[inline(always)]
pub(crate) fn divide_alpha_row(src_row: &[U8x2], dst_row: &mut [U8x2]) {
    src_row
        .iter()
        .zip(dst_row)
        .for_each(|(src_pixel, dst_pixel)| {
            let components: [u8; 2] = src_pixel.0;
            let alpha = components[1];
            let recip_alpha = RECIP_ALPHA[alpha as usize];
            dst_pixel.0 = [div_and_clip(components[0], recip_alpha), alpha];
        });
}
