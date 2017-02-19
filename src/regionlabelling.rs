//! Functions for finding and labelling connected components of an image.

use image::{
    GenericImage,
    ImageBuffer,
    Luma
};

use definitions::{
    VecBuffer
};

use unionfind::{
    DisjointSetForest
};

use std::{
    cmp
};

/// Determines which neighbors of a pixel we consider
/// to be connected to it.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Connectivity {
    /// A pixel is connected to its N, S, E and W neighbors.
    Four,
    /// A pixel is connected to all of its neighbors.
    Eight
}

/// Returns an image of the same size as the input, where each pixel
/// is labelled by the connected foreground component it belongs to,
/// or 0 if it's in the background. Input pixels are treated as belonging
/// to the background if and only if they are equal to the provided background pixel.
pub fn connected_components<I>(image: &I, conn: Connectivity, background: I::Pixel) -> VecBuffer<Luma<u32>>
    where I: GenericImage,
          I::Pixel: Eq
{
    let (width, height) = image.dimensions();
    let mut out = ImageBuffer::new(width, height);

    // TODO: add macro to abandon early if either dimension is zero
    if width == 0 || height == 0 {
        return out;
    }

    let image_size = (width * height) as usize;
    let mut forest = DisjointSetForest::new(image_size);
    let mut adj_labels = [0u32; 4];
    let mut next_label = 1;

    for y in 0..height {
        for x in 0..width {
            let current = unsafe { image.unsafe_get_pixel(x, y) };
            if current == background {
                continue;
            }

            let mut num_adj = 0;

            if x > 0 {
                // West
                let pixel = unsafe { image.unsafe_get_pixel(x - 1, y) };
                if pixel == current {
                    let label = unsafe { out.unsafe_get_pixel(x - 1, y)[0] };
                    adj_labels[num_adj] = label;
                    num_adj += 1;
                }
            }

            if y > 0 {
                // North
                let pixel = unsafe { image.unsafe_get_pixel(x, y - 1) };
                if pixel == current {
                    let label = unsafe { out.unsafe_get_pixel(x, y - 1)[0] };
                    adj_labels[num_adj] = label;
                    num_adj += 1;
                }

                if conn == Connectivity::Eight {
                    if x > 0 {
                        // North West
                        let pixel = unsafe { image.unsafe_get_pixel(x - 1, y - 1) };
                        if pixel == current {
                            let label = unsafe { out.unsafe_get_pixel(x - 1, y - 1)[0] };
                            adj_labels[num_adj] = label;
                            num_adj += 1;
                        }
                    }
                    if x < width - 1 {
                        // North East
                        let pixel = unsafe { image.unsafe_get_pixel(x + 1, y - 1) };
                        if pixel == current {
                            let label = unsafe { out.unsafe_get_pixel(x + 1, y - 1)[0] };
                            adj_labels[num_adj] = label;
                            num_adj += 1;
                        }
                    }
                }
            }

            if num_adj == 0 {
                unsafe { out.unsafe_put_pixel(x, y, Luma([next_label])); }
                next_label += 1;
            }
            else {
                let mut min_label = u32::max_value();
                for n in 0..num_adj {
                    min_label = cmp::min(min_label, adj_labels[n]);
                }
                unsafe { out.unsafe_put_pixel(x, y, Luma([min_label])); }
                for n in 0..num_adj {
                    forest.union(min_label as usize, adj_labels[n] as usize);
                }
            }
        }
    }

    // Make components start at 1
    let mut output_labels = vec![0u32; image_size];
    let mut count = 1;

    unsafe {
        for y in 0..height {
            for x in 0..width {
                let label = {
                    if image.unsafe_get_pixel(x, y) == background {
                        continue;
                    }
                    out.unsafe_get_pixel(x, y)[0]
                };
                let root = forest.root(label as usize);
                let mut output_label = *output_labels.get_unchecked(root);
                if output_label < 1 {
                    output_label = count;
                    count += 1;
                }
                *output_labels.get_unchecked_mut(root) = output_label;
                out.unsafe_put_pixel(x, y, Luma([output_label]));
            }
        }
    }

    out
}

#[cfg(test)]
mod test {

    use super::{
        connected_components
    };
    use super::Connectivity::{
        Four,
        Eight
    };
    use definitions::{
        HasBlack,
        HasWhite
    };
    use image::{
        GrayImage,
        ImageBuffer,
        Luma
    };
    use test;

    #[test]
    fn test_connected_components_four() {
        let image: GrayImage = ImageBuffer::from_raw(4, 4, vec![
            1, 0, 2, 1,
            0, 1, 1, 0,
            0, 0, 0, 0,
            0, 0, 0, 1]).unwrap();

        let expected: ImageBuffer<Luma<u32>, Vec<u32>>
            = ImageBuffer::from_raw(4, 4, vec![
                1, 0, 2, 3,
                0, 4, 4, 0,
                0, 0, 0, 0,
                0, 0, 0, 5]).unwrap();

        let labelled = connected_components(&image, Four, Luma::black());
        assert_pixels_eq!(labelled, expected);
    }

    #[test]
    fn test_connected_components_eight() {
        let image: GrayImage = ImageBuffer::from_raw(4, 4, vec![
            1, 0, 2, 1,
            0, 1, 1, 0,
            0, 0, 0, 0,
            0, 0, 0, 1]).unwrap();

        let expected: ImageBuffer<Luma<u32>, Vec<u32>>
            = ImageBuffer::from_raw(4, 4, vec![
                1, 0, 2, 1,
                0, 1, 1, 0,
                0, 0, 0, 0,
                0, 0, 0, 3]).unwrap();

        let labelled = connected_components(&image, Eight, Luma::black());
        assert_pixels_eq!(labelled, expected);
    }

    #[test]
    fn test_connected_components_eight_white_background() {
        let image: GrayImage = ImageBuffer::from_raw(4, 4, vec![
              1, 255,   2,   1,
            255,   1,   1, 255,
            255, 255, 255, 255,
            255, 255, 255,   1]).unwrap();

        let expected: ImageBuffer<Luma<u32>, Vec<u32>>
            = ImageBuffer::from_raw(4, 4, vec![
                1, 0, 2, 1,
                0, 1, 1, 0,
                0, 0, 0, 0,
                0, 0, 0, 3]).unwrap();

        let labelled = connected_components(&image, Eight, Luma::white());
        assert_pixels_eq!(labelled, expected);
    }

    // One huge component with eight-way connectivity, loads of
    // isolated components with four-way conectivity.
    fn chessboard(width: u32, height: u32) -> GrayImage {
        ImageBuffer::from_fn(width, height, |x, y| {
            if (x + y) % 2 == 0 { return Luma([255u8]); }
            else { return Luma([0u8]); }
        })
    }

    #[test]
    fn test_connected_components_eight_chessboard() {
        let image = chessboard(30, 30);
        let components = connected_components(&image, Eight, Luma::black());
        let max_component = components.pixels().map(|p| p[0]).max();
        assert_eq!(max_component, Some(1u32));
    }

    #[test]
    fn test_connected_components_four_chessboard() {
        let image = chessboard(30, 30);
        let components = connected_components(&image, Four, Luma::black());
        let max_component = components.pixels().map(|p| p[0]).max();
        assert_eq!(max_component, Some(450u32));
    }

    #[bench]
    fn bench_connected_components_eight_chessboard(b: &mut test::Bencher) {
        let image = chessboard(300, 300);
        b.iter(|| {
            let components = connected_components(&image, Eight, Luma::black());
            test::black_box(components);
            });
    }

    #[bench]
    fn bench_connected_components_four_chessboard(b: &mut test::Bencher) {
        let image = chessboard(300, 300);
        b.iter(|| {
            let components = connected_components(&image, Four, Luma::black());
            test::black_box(components);
            });
    }
}
