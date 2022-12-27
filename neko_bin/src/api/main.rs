// Copyright 2022 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let port = neko_env::HTTP_PORT.to_owned();
  HttpServer::new(|| App::new())
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
