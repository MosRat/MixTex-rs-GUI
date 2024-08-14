use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageReader};
use image::imageops::FilterType;


struct Config {
    width: u32,
    height: u32,
    rescale_factor: f32,
    norm_mean: f32,
    norm_std: f32,
}

const CONFIG: Config = Config {
    width: 448,
    height: 448,
    rescale_factor: 0.00392156862745098,
    norm_mean: 0.5,
    norm_std: 0.5,
};

pub fn resize(img: DynamicImage) -> DynamicImage {
    img.resize_exact(CONFIG.width, CONFIG.height, FilterType::CatmullRom)
}

pub fn rescale_and_normalize(img: DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();
    let mut rescaled_img = DynamicImage::new_rgb32f(width, height);
    let  pixel_ref = rescaled_img.as_mut_rgb32f().unwrap();
    img.into_rgba8().enumerate_pixels().for_each(|(x, y, pixel)| {
        let rescaled_pixel = image::Rgb([
            (pixel[0] as f32 * CONFIG.rescale_factor - CONFIG.norm_mean) / CONFIG.norm_std,
            (pixel[1] as f32 * CONFIG.rescale_factor - CONFIG.norm_mean) / CONFIG.norm_std,
            (pixel[2] as f32 * CONFIG.rescale_factor - CONFIG.norm_mean) / CONFIG.norm_std,
        ]);
        pixel_ref.put_pixel(x, y, rescaled_pixel);
    });


    rescaled_img
}
// struct SharedArrayPtr {
//     ptr: *mut [f32; (3 * CONFIG.width * CONFIG.height) as usize],
// }
//
// unsafe impl Sync for SharedArrayPtr {}
// unsafe impl Send for SharedArrayPtr {}
pub fn preprocess(path: impl AsRef<Path>) -> Vec<f32> {
    let img = rescale_and_normalize(resize(
        ImageReader::open(path)
            .unwrap()
            .decode()
            .unwrap()
    ));

    // let res = std::cell::UnsafeCell::new([0.; (3 * CONFIG.width * CONFIG.height) as usize]);
    let mut res = vec![0.0f32; (3 * CONFIG.width * CONFIG.height) as usize];
    // let res = Box::new([0.0f32; (3 * CONFIG.width * CONFIG.height) as usize]);
    // let res_ptr = SharedArrayPtr { ptr: res.as_ptr() as *mut [f32; (3 * CONFIG.width * CONFIG.height) as usize] };
    // let res_arc = Arc::new(&res_ptr);
    let channel_size = (CONFIG.width * CONFIG.height) as usize;
    let _ = img.into_rgb32f().enumerate_pixels().for_each(|(x, y, pixel)| {
        for i in 0..3 {
            let index = i * channel_size + (y * CONFIG.width + x) as usize;
            res[index] = pixel[i];
        }
    });

    res
    // Vec::from_p(res)
    // img.into_rgb32f().into_vec()
}