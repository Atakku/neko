// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::*;
use axum::{Router, Server};
use derivative::Derivative;
use futures::future::BoxFuture;
use std::net::Ipv4Addr;

#[derive(Derivative)]
#[derivative(Default)]
pub struct Axum {
  pub routes: Vec<fn(Router) -> BoxFuture<'static, Res<Router>>>,
  #[derivative(Default(value = "8080"))]
  pub port: u16,
}


impl Module for Axum {
  async fn init(&mut self, fw: &mut Framework) -> R {
    fw.runtime.push(|m| {
      let axum = m.take::<Self>()?;
      Ok(Box::pin(async move {
        let mut router = Router::new();
        for route in axum.routes {
          router = route(router).await?;
        }
        Ok(Some(tokio::spawn(async move {
          Server::bind(&(Ipv4Addr::UNSPECIFIED, axum.port).into())
            .serve(router.into_make_service())
            .await?;
          Ok(())
        })))
      }))
    });
    Ok(())
  }
}
