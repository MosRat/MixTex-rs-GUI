/*
 * Copyright (c) 2024. MosRat
 * All rights reserved.
 *
 * Project: fast-writer
 * File Name: api.rs
 * Author: MosRat (work@whl.moe)
 * Description:
 */
use anyhow::{anyhow, Result};
use image::{ImageBuffer, ImageFormat, Rgba};
use serde_json::Value;
use std::io::Cursor;
use tauri_plugin_http::reqwest::{header::HeaderMap, multipart, Client};

pub fn encode_rgba_to_png(rgba_data: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    // 检查输入数据长度是否匹配宽高
    if rgba_data.len() != (width * height * 4) as usize {
        return Err(anyhow!("Invalid RGBA data length"));
    }

    // 创建图像缓冲区
    let img: ImageBuffer<Rgba<u8>, &[u8]> = ImageBuffer::from_raw(width, height, rgba_data)
        .ok_or(anyhow!("Failed to create image buffer"))?;

    // 创建一个内存缓冲区来存储PNG数据
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    // 将图像编码为PNG并写入缓冲区
    img.write_to(&mut cursor, ImageFormat::Png)?;

    Ok(buffer)
}

pub async fn simple_latex(img: Vec<u8>, token: &str) -> Result<String, String> {
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| e.to_string())?;

    let mut headers = HeaderMap::new();
    headers.insert("token", token.parse().unwrap());

    let part = multipart::Part::bytes(img).file_name("file.png");
    let form = multipart::Form::new().part("file", part);

    let res: Value = client
        .post("https://server.simpletex.cn/api/latex_ocr/v2")
        .headers(headers)
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    if res
        .get("status")
        .ok_or("cant get json value!")?
        .as_bool()
        .ok_or("cant get json value!")?
        == true
    {
        Ok(res
            .get("res")
            .ok_or("cant get json value!")?
            .get("latex")
            .ok_or("cant get json value!")?
            .as_str()
            .ok_or("cant get json value!")?
            .to_string())
    } else {
        Err(format!("Fail to {:}", res))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    #[tokio::test]
    async fn test_simple_tex() {
        let mut img = std::fs::read(r#"E:\WorkSpace\RustProjects\fast-writer\img.png"#).unwrap();
        println!("{:}", simple_latex(img).await.unwrap());
    }
}
