// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::*;
use axum::{Router, Server};
use std::net::Ipv4Addr;

pub struct Axum {
  pub routes: Vec<fn(Router) -> Router>,
  pub port: u16,
}

impl Default for Axum {
  fn default() -> Self {
    Self {
      routes: vec![],
      port: 8080,
    }
  }
}

impl Module for Axum {
  fn init(&mut self, fw: &mut Framework) -> R {
    fw.runtime.push(|m, rt| {
      let axum = m.take::<Self>()?;
      Ok(Box::pin(async move {
        Ok(Some(tokio::spawn(async move {
          let mut router = Router::new();
          for route in axum.routes {
            router = route(router);
          }
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
