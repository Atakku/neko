use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use mime_guess::from_path;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../dist/"]
struct Dist;

fn handle_embedded_file(path: &str) -> HttpResponse {
  match Dist::get(path) {
    Some(content) => HttpResponse::Ok()
      .content_type(from_path(path).first_or_octet_stream().as_ref())
      .body(content.data.into_owned()),
    None => handle_embedded_file("index.html"),
  }
}

#[actix_web::get("{_:.*}")]
async fn dist(path: web::Path<String>) -> impl Responder {
  handle_embedded_file(path.as_str())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let port = neko_env::HTTP_PORT.to_owned();
  HttpServer::new(|| App::new().service(dist))
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
