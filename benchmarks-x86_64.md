<!-- introduction start -->

## Benchmarks of fast_image_resize crate for x86_64 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 4000 MHz
- Ubuntu 24.04 (linux 6.17.0)
- Rust 1.96.0
- criterion = "0.7.0"
- fast_image_resize = "6.0.1"

Other libraries used to compare of resizing speed:

- image = "0.25.6" (<https://crates.io/crates/image>)
- resize = "0.8.9" (<https://crates.io/crates/resize>, single-threaded mode)
- libvips = "8.15.1" (single-threaded mode)

Resize algorithms:

- Nearest
- Box - convolution with minimal kernel size 1x1 px
- Bilinear - convolution with minimal kernel size 2x2 px
- Bicubic (CatmullRom) - convolution with minimal kernel size 4x4 px
- Lanczos3 - convolution with minimal kernel size 6x6 px

<!-- introduction end -->

<!-- bench_compare_rgb start -->

### Resize RGB8 image (U8x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in the table mean the duration of the image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  28.92  |   -   |  78.41   | 127.38  |  173.98  |
| resize     |  8.49   | 24.35 |  48.44   |  92.29  |  137.61  |
| libvips    |  2.41   | 61.63 |   5.67   |  9.76   |  16.07   |
| fir rust   |  0.28   | 10.72 |  15.67   |  26.08  |  37.15   |
| fir sse4.1 |  0.28   | 3.74  |   5.64   |  10.28  |  15.98   |
| fir avx2   |  0.28   | 2.77  |   4.16   |  7.37   |  14.35   |

<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean the duration of the image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize     |  13.97  | 44.38  |  88.03   | 150.33  |  216.80  |
| libvips    |  4.19   | 169.24 |  137.35  | 228.62  |  329.39  |
| fir rust   |  0.20   | 20.57  |  25.80   |  37.26  |  50.53   |
| fir sse4.1 |  0.20   | 10.17  |  13.77   |  18.20  |  25.35   |
| fir avx2   |  0.20   |  7.41  |   9.02   |  13.46  |  25.45   |

<!-- bench_compare_rgba end -->

<!-- bench_compare_l start -->

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in the table mean the duration of the image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  26.33  |   -   |  57.97   |  86.30  |  113.91  |
| resize     |  6.59   | 11.63 |  21.04   |  45.28  |  68.73   |
| libvips    |  2.66   | 24.92 |   6.83   |  9.81   |  12.72   |
| fir rust   |  0.16   | 4.34  |   5.46   |  8.73   |  12.09   |
| fir sse4.1 |  0.16   | 1.66  |   2.25   |  3.67   |   5.89   |
| fir avx2   |  0.16   | 1.92  |   2.04   |  3.16   |   4.51   |

<!-- bench_compare_l end -->

<!-- bench_compare_la start -->

### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with an alpha channel (two bytes per pixel).
- Numbers in the table mean the duration of the image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| libvips    |  3.73   | 94.13 |  76.50   | 123.09  |  165.76  |
| fir rust   |  0.18   | 17.96 |  20.33   |  26.63  |  32.50   |
| fir sse4.1 |  0.18   | 6.24  |   7.27   |  10.11  |  13.94   |
| fir avx2   |  0.18   | 4.23  |   5.10   |  7.01   |   9.67   |

<!-- bench_compare_la end -->

<!-- bench_compare_rgb16 start -->

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in the table mean the duration of the image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  29.05  |   -   |  86.11   | 142.76  |  195.46  |
| resize     |  7.11   | 21.55 |  41.21   |  80.56  |  120.40  |
| libvips    |  14.16  | 95.93 |  65.56   | 130.57  |  174.91  |
| fir rust   |  0.34   | 26.34 |  41.24   |  70.55  |  102.27  |
| fir sse4.1 |  0.34   | 15.83 |  23.98   |  40.83  |  58.45   |
| fir avx2   |  0.34   | 12.74 |  17.92   |  28.22  |  36.64   |

<!-- bench_compare_rgb16 end -->

<!-- bench_compare_rgba16 start -->

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean the duration of the image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize     |  13.98  | 44.66  |  88.98   | 150.55  |  215.91  |
| libvips    |  21.38  | 182.44 |  151.63  | 245.07  |  344.34  |
| fir rust   |  0.40   | 56.25  |  75.07   | 112.56  |  155.00  |
| fir sse4.1 |  0.40   | 32.20  |  43.46   |  67.05  |  91.03   |
| fir avx2   |  0.40   | 21.16  |  26.25   |  37.12  |  48.80   |

<!-- bench_compare_rgba16 end -->

<!-- bench_compare_l16 start -->

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in the table mean the duration of the image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  27.19  |   -   |  60.30   |  90.07  |  120.65  |
| resize     |  5.23   | 11.68 |  21.32   |  45.53  |  69.43   |
| libvips    |  5.69   | 34.84 |  23.63   |  43.63  |  59.30   |
| fir rust   |  0.17   | 13.51 |  18.48   |  31.83  |  43.04   |
| fir sse4.1 |  0.17   | 5.65  |   7.69   |  13.75  |  19.97   |
| fir avx2   |  0.17   | 5.52  |   6.26   |  9.36   |  13.87   |

<!-- bench_compare_l16 end -->

<!-- bench_compare_la16 start -->

### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with an alpha channel (four bytes per pixel).
- Numbers in the table mean the duration of the image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| libvips    |  11.21  | 105.18 |  86.35   | 133.43  |  176.69  |
| fir rust   |  0.19   | 29.30  |  36.05   |  56.42  |  76.58   |
| fir sse4.1 |  0.19   | 15.16  |  21.12   |  33.81  |  47.86   |
| fir avx2   |  0.19   | 11.73  |  14.43   |  21.45  |  28.77   |

<!-- bench_compare_la16 end -->

<!-- bench_compare_l32f start -->

### Resize L32F image (F32) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in the table mean the duration of the image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  23.98  |   -   |  48.61   |  76.35  |  102.23  |
| resize     |  5.09   | 9.47  |  14.45   |  31.12  |  47.11   |
| libvips    |  4.66   | 34.02 |  23.58   |  45.85  |  64.97   |
| fir rust   |  0.19   | 7.34  |  11.99   |  26.38  |  39.36   |
| fir sse4.1 |  0.19   | 4.66  |   7.38   |  13.04  |  19.01   |
| fir avx2   |  0.19   | 4.34  |   5.54   |  8.73   |  12.47   |

<!-- bench_compare_l32f end -->

<!-- bench_compare_la32f start -->

### Resize LA32F (luma with alpha channel) image (F32x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with an alpha channel (two `f32` values per pixel).
- Numbers in the table mean the duration of the image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| libvips    |  10.63  | 92.48 |  75.19   | 121.33  |  162.76  |
| fir rust   |  0.38   | 21.69 |  29.09   |  47.93  |  70.81   |
| fir sse4.1 |  0.38   | 17.34 |  22.56   |  33.18  |  44.69   |
| fir avx2   |  0.38   | 16.11 |  18.34   |  25.32  |  31.96   |

<!-- bench_compare_la32f end -->

<!-- bench_compare_rgb32f start -->

### Resize RGB32F image (F32x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB32F image.
- Numbers in the table mean the duration of the image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  25.78  |   -   |  62.70   | 106.83  |  141.64  |
| resize     |  9.06   | 13.80 |  23.82   |  48.45  |  72.45   |
| libvips    |  10.77  | 91.73 |  65.25   | 136.13  |  189.49  |
| fir rust   |  0.89   | 14.21 |  25.12   |  48.44  |  73.92   |
| fir sse4.1 |  0.89   | 12.21 |  20.15   |  35.74  |  52.87   |
| fir avx2   |  0.89   | 9.77  |  14.02   |  23.58  |  32.99   |

<!-- bench_compare_rgb32f end -->


<!-- bench_compare_rgba32f start -->

### Resize RGBA32F image (F32x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean the duration of the image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support multiplying and dividing by alpha channel
  for this pixel format.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| libvips    |  19.95  | 154.11 |  126.57  | 210.63  |  310.66  |
| fir rust   |  1.05   | 34.90  |  44.55   |  69.28  |  93.01   |
| fir sse4.1 |  1.05   | 31.93  |  41.31   |  62.85  |  84.26   |
| fir avx2   |  1.05   | 29.11  |  31.47   |  42.95  |  55.41   |

<!-- bench_compare_rgba32f end -->
