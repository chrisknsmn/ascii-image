use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_files as fs;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Result};
use futures::{StreamExt, TryStreamExt};
use image::{DynamicImage, GenericImageView, Pixel};
use serde::Deserialize;
use std::io::Write;

#[derive(Deserialize)]
struct AsciiParams {
    brightness: f32,
    contrast: f32,
    width: u32,
}

#[get("/")]
async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}

#[post("/upload")]
async fn upload(mut payload: Multipart, params: web::Query<AsciiParams>) -> Result<HttpResponse> {
    while let Ok(Some(mut field)) = payload.try_next().await {
        if field.name() == "image" {
            let mut bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                bytes.write_all(&data)?;
            }

            if let Ok(img) = image::load_from_memory(&bytes) {
                let ascii = image_to_ascii(img, params.0);
                return Ok(HttpResponse::Ok().body(ascii));
            }
        }
    }
    Ok(HttpResponse::BadRequest().body("No image found"))
}

fn image_to_ascii(img: DynamicImage, params: AsciiParams) -> String {
    let (orig_width, orig_height) = img.dimensions();
    let aspect_ratio = orig_height as f32 / orig_width as f32;

    // Adjust height based on font aspect ratio (e.g., 2.0 for most monospace fonts)
    let font_aspect_ratio = 1.6;
    let height = (params.width as f32 * aspect_ratio / font_aspect_ratio) as u32;

    let img = img.resize_exact(params.width, height, image::imageops::FilterType::Nearest);

    let ascii_chars = vec!["@","#","S","%","?","*","+",";",":",",","."," "];
    let mut ascii_art = String::new();
    
    for y in 0..img.height() {
        for x in 0..img.width() {
            let pixel = img.get_pixel(x, y);
            let mut intensity = pixel.to_luma()[0] as f32 / 255.0;
            
            // Apply brightness
            intensity = (intensity + (params.brightness - 50.0) / 100.0).max(0.0).min(1.0);

            // Apply contrast
            let contrast = (params.contrast - 50.0) / 50.0;
            intensity = (((intensity - 0.5) * (1.0 + contrast)) + 0.5).max(0.0).min(1.0);
            
            let char_index = (intensity * (ascii_chars.len() - 1) as f32) as usize;
            ascii_art.push_str(ascii_chars[char_index]);
        }
        ascii_art.push('\n');
    }
    ascii_art
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        App::new()
            .service(index)  // Register the index route
            .service(upload) // Register the upload route
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?;

    println!("🚀 Server running at http://127.0.0.1:8080");
    
    server.run().await
}

