use actix_web::{get, web, HttpResponse};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "resizer", about = "A very fixed-function image proxy")]
struct Opt {
    #[structopt(short, long, default_value = "8000")]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};
    let opts = Opt::from_args();
    let bind_addr = std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(0, 0, 0, 0), opts.port);
    HttpServer::new(move || App::new().service(get_image))
        .bind(bind_addr)?
        .workers(1)
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
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let body = client
        .get(url)
        .send()
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
