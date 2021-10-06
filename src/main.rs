use actix_web::{body, web, App, HttpResponse, HttpServer, get};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .service(get_image)
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}

#[get("/{cam}")]
pub async fn get_image(what: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    println!("Request");
    let camno = what.into_inner();
    if camno > 10 {
        return Err(actix_web::error::ErrorBadRequest("invalid camera"));
    }
    let url = format!("http://192.168.10{}.2:8000/", camno);
    let body = reqwest::get(url)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("{}", e)))?
        .bytes()
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("{}", e)))?;
    let img = image::io::Reader::new(std::io::Cursor::new(body)).with_guessed_format().expect("Cursor IO never fails").decode().unwrap();
    let img = img.resize(820, 616, image::imageops::FilterType::Nearest);
    let mut b: Vec<u8> = Vec::new();
    img.write_to(&mut b, image::ImageOutputFormat::Jpeg(50)).unwrap();
    Ok(HttpResponse::Ok()
                .content_type("image/jpeg")
                .body(body::Body::from_slice(&b)))
}
