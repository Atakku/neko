// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::*;
use futures::future::join_all;
use poise::{
  serenity_prelude::{Context as SCtx, GatewayIntents},
  BoxFuture, Command, Context, Event, FrameworkContext, FrameworkOptions,
};

pub type Fw = poise::Framework<StateLock, Err>;
pub type FwCtx<'a> = FrameworkContext<'a, StateLock, Err>;
pub type Ctx<'a> = Context<'a, StateLock, Err>;
pub type Cmd = Command<StateLock, Err>;
pub type EventHandler =
  for<'a> fn(&'a SCtx, &'a Event<'a>, FwCtx<'a>, &'a StateLock) -> BoxFuture<'a, R>;

/// Poise wrapper module, to let other modules add commands and subscribe to events easily
pub struct Poise {
  token: String,
  pub intents: GatewayIntents,
  pub commands: Vec<Cmd>,
  pub event_handlers: Vec<EventHandler>,
}

impl Default for Poise {
  fn default() -> Self {
    Self {
      token: std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is not present"),
      intents: GatewayIntents::empty(),
      commands: vec![],
      event_handlers: vec![],
    }
  }
}

impl Module for Poise {
  fn init(&self, fw: &mut Framework) -> R {
    fw.runtime.push(|mds, rt| {
      let poise = mds.take::<Self>()?;
      Ok(Box::pin(async move {
        rt.write().await.put(poise.event_handlers);
        Ok(Some(tokio::spawn(async move {
          Fw::builder()
            .token(poise.token)
            .intents(poise.intents)
            .setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(rt) }))
            .options(FrameworkOptions {
              commands: poise.commands,
              event_handler: |ctx, e, fctx, rt| {
                Box::pin(async move {
                  let ehs = rt.read().await.borrow::<Vec<EventHandler>>()?.clone();
                  join_all(ehs.iter().map(|eh| (eh)(ctx, e, fctx, rt))).await;
                  Ok(())
                })
              },
              ..Default::default()
            })
            .run()
            .await?;
          Ok(())
        })))
      }))
    });
    Ok(())
  }
}
