use actix_web::{get, web, App, HttpResponse, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || App::new().service(get_image))
        .bind("0.0.0.0:8000")?
        .run()
        .await
}

#[get("/{cam}")]
pub async fn get_image(what: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    let camno = what.into_inner();
    if camno > 9 {
        return Err(actix_web::error::ErrorBadRequest("invalid camera"));
    }
    let url = format!("http://192.168.10{}.2:8000/", camno);
    let body = reqwest::get(url)
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
        .bytes()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    let img = image::io::Reader::new(std::io::Cursor::new(body))
        .with_guessed_format()
        .expect("Cursor IO never fails")
        .decode()
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let img = img.resize(820, 616, image::imageops::FilterType::Nearest);
    let mut b: Vec<u8> = Vec::new();
    img.write_to(&mut b, image::ImageOutputFormat::Jpeg(50))
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().content_type("image/jpeg").body(b))
}
