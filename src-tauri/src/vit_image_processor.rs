use std::path::Path;

use image::{load_from_memory, DynamicImage, GenericImageView, ImageReader};
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

pub fn preprocess(path: impl AsRef<Path>) -> Result<Vec<f32>,String> {

    let img = rescale_and_normalize(resize(
        ImageReader::open(path)
            .map_err(|err| err.to_string())?
            .decode()
            .map_err(|err| err.to_string())?
    ));

    let mut res = vec![0.0f32; (3 * CONFIG.width * CONFIG.height) as usize];

    let channel_size = (CONFIG.width * CONFIG.height) as usize;
    let _ = img.into_rgb32f().enumerate_pixels().for_each(|(x, y, pixel)| {
        for i in 0..3 {
            let index = i * channel_size + (y * CONFIG.width + x) as usize;
            res[index] = pixel[i];
        }
    });

   Ok(res)
}

pub fn preprocess_from_memory(data: &[u8]) -> Result<Vec<f32>,String> {

    let img = rescale_and_normalize(resize(
        load_from_memory(data).map_err(|err| err.to_string())?
    ));

    let mut res = vec![0.0f32; (3 * CONFIG.width * CONFIG.height) as usize];

    let channel_size = (CONFIG.width * CONFIG.height) as usize;
    let _ = img.into_rgb32f().enumerate_pixels().for_each(|(x, y, pixel)| {
        for i in 0..3 {
            let index = i * channel_size + (y * CONFIG.width + x) as usize;
            res[index] = pixel[i];
        }
    });
    Ok(res)
}