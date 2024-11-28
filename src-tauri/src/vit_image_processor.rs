use std::path::Path;

use image::imageops::FilterType;
use image::{
    load_from_memory, DynamicImage, GenericImage, GenericImageView, ImageReader, Rgb, Rgb32FImage,
};

struct Config {
    pub width: u32,
    pub height: u32,
    pub rescale_factor: f32,
    pub norm_mean: f32,
    pub norm_std: f32,
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

pub fn padding(img: DynamicImage) -> DynamicImage {
    let mut background = DynamicImage::from(Rgb32FImage::from_pixel(
        CONFIG.width,
        CONFIG.height,
        Rgb::from([255_f32, 255_f32, 255_f32]),
    ));
    if img.width() <= CONFIG.width && img.height() <= CONFIG.height {
        background
            .sub_image(
                (CONFIG.width - img.width()) / 2,
                (CONFIG.height - img.height()) / 2,
                img.width(),
                img.height(),
            )
            .copy_from(&img, 0, 0)
            .expect("fail!");
    } else {
        let scale = (CONFIG.width as f32 / img.width() as f32)
            .min(CONFIG.height as f32 / img.height() as f32);
        let img_resize = img.resize_exact(
            (img.width() as f32 * scale) as u32,
            (img.height() as f32 * scale) as u32,
            FilterType::Lanczos3,
        );
        background
            .sub_image(
                (CONFIG.width - img_resize.width()) / 2,
                (CONFIG.height - img_resize.height()) / 2,
                img_resize.width(),
                img_resize.height(),
            )
            .copy_from(&img_resize, 0, 0)
            .expect("fail!");
    }
    // background.to_rgba8().save("./img.png").expect("fail to padding!");
    // info!("{:?} {:?} {:?}",background.color(),background.width(),background.height());
    DynamicImage::from(background.to_rgba8())
}

pub fn rescale_and_normalize(img: DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();
    let mut rescaled_img = DynamicImage::new_rgb32f(width, height);
    let pixel_ref = rescaled_img.as_mut_rgb32f().unwrap();
    img.into_rgba8()
        .enumerate_pixels()
        .for_each(|(x, y, pixel)| {
            let rescaled_pixel = image::Rgb([
                (pixel[0] as f32 * CONFIG.rescale_factor - CONFIG.norm_mean) / CONFIG.norm_std,
                (pixel[1] as f32 * CONFIG.rescale_factor - CONFIG.norm_mean) / CONFIG.norm_std,
                (pixel[2] as f32 * CONFIG.rescale_factor - CONFIG.norm_mean) / CONFIG.norm_std,
            ]);
            pixel_ref.put_pixel(x, y, rescaled_pixel);
        });

    rescaled_img
}

pub fn preprocess(path: impl AsRef<Path>) -> Result<Vec<f32>, String> {
    let img = rescale_and_normalize(padding(
        ImageReader::open(path)
            .map_err(|err| err.to_string())?
            .decode()
            .map_err(|err| err.to_string())?,
    ));

    let mut res = vec![0.0f32; (3 * CONFIG.width * CONFIG.height) as usize];

    let channel_size = (CONFIG.width * CONFIG.height) as usize;
    let _ = img
        .into_rgb32f()
        .enumerate_pixels()
        .for_each(|(x, y, pixel)| {
            for i in 0..3 {
                let index = i * channel_size + (y * CONFIG.width + x) as usize;
                res[index] = pixel[i];
            }
        });

    Ok(res)
}

pub fn preprocess_from_memory(data: &[u8]) -> Result<Vec<f32>, String> {
    let img = rescale_and_normalize(resize(
        load_from_memory(data).map_err(|err| err.to_string())?,
    ));

    let mut res = vec![0.0f32; (3 * CONFIG.width * CONFIG.height) as usize];

    let channel_size = (CONFIG.width * CONFIG.height) as usize;
    let _ = img
        .into_rgb32f()
        .enumerate_pixels()
        .for_each(|(x, y, pixel)| {
            for i in 0..3 {
                let index = i * channel_size + (y * CONFIG.width + x) as usize;
                res[index] = pixel[i];
            }
        });
    Ok(res)
}

pub fn preprocess_from_image(img: DynamicImage) -> Result<Vec<f32>, String> {
    let img = rescale_and_normalize(resize(img));

    let mut res = vec![0.0f32; (3 * CONFIG.width * CONFIG.height) as usize];

    let channel_size = (CONFIG.width * CONFIG.height) as usize;
    let _ = img
        .into_rgb32f()
        .enumerate_pixels()
        .for_each(|(x, y, pixel)| {
            for i in 0..3 {
                let index = i * channel_size + (y * CONFIG.width + x) as usize;
                res[index] = pixel[i];
            }
        });
    Ok(res)
}

pub fn preprocess_from_rgb_array(img: &[u8], w: u32, h: u32) -> Result<Vec<f32>, String> {
    let img = rescale_and_normalize(resize(DynamicImage::from(
        image::RgbaImage::from_raw(w, h, img.into()).ok_or("Read image fail!")?,
    )));

    let mut res = vec![0.0f32; (3 * CONFIG.width * CONFIG.height) as usize];

    let channel_size = (CONFIG.width * CONFIG.height) as usize;
    let _ = img
        .into_rgb32f()
        .enumerate_pixels()
        .for_each(|(x, y, pixel)| {
            for i in 0..3 {
                let index = i * channel_size + (y * CONFIG.width + x) as usize;
                res[index] = pixel[i];
            }
        });
    Ok(res)
}
