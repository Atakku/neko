// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{core::*, modules::poise::Poise};
use poise::{
  serenity_prelude::{
    ButtonStyle, EmojiId,
    Interaction::MessageComponent, ReactionType, RoleId,
  },
  BoxFuture, Event,
};

/// Module with femboy.tv discord server functionality
pub struct Starboard;

impl Module for Starboard {
  fn init(&mut self, fw: &mut Framework) -> R {
    let poise = fw.req_module::<Poise>()?;
    poise.event_handlers.push(event_handler);
    Ok(())
  }
}

fn event_handler<'a>(
  c: &'a poise::serenity_prelude::Context,
  event: &'a Event<'a>,
) -> BoxFuture<'a, R> {
  Box::pin(async move {
    use Event::*;
    match event {
      ReactionAdd { add_reaction } => {
        
      }
      ReactionRemove { removed_reaction } => {

      }
      _ => {}
    }
    Ok(())
  })
}
