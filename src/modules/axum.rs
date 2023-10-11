// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::*;
use axum::{Router, Server};
use futures::future::BoxFuture;
use std::net::Ipv4Addr;

pub type Route = fn(Router) -> BoxFuture<'static, Res<Router>>;

module! {
  Axum {
    routes: Vec<Route>,
    port: u16 = 8080,
  }

  fn init(fw) {
    rt!(fw, |axum| {
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
    });
  }

  // TODO: solve the issue with overlap
  pub fn add_route(&mut self, route: Route) {
    self.routes.push(route);
  }
}

