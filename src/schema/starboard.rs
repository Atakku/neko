// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sea_query::Iden;

#[derive(Iden)]
#[iden(rename = "starboard_messages")]
pub enum Messages {
  Table,
  RepostId,
  ChannelId,
  MessageId,
}

#[derive(Iden)]
#[iden(rename = "starboard_reactions")]
pub enum Reactions {
  Table,
  MessageId,
  Reaction,
  Count
}
