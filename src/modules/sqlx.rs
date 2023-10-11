// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sqlx::{postgres::PgPoolOptions, PgPool};

once_cell!(db, POOL: PgPool);

module! {
  Postgres {
    db_url: String = env!("DATABASE_URL"),
    options: PgPoolOptions,
  }

  fn init(fw) {
    rt!(fw, |postgres| {
      POOL.set(postgres.options.connect(&postgres.db_url).await?)?;
      //sqlx::migrate!("./sql").run(db()).await.unwrap();
      Ok(None)
    });
  }
}

//TODO: generate init query
/// Declare a new sea_query schema
macro_rules! schema {
  ($(
    #[table($table:literal)]
    //$(#[alias($alias:ident)])*
    $vis:vis enum $ident:ident {
      $($field:ident),*$(,)?
    }
  )*) => {
    $(
      #[derive(sea_query::Iden)]
      #[iden(rename = $table)]
      #[allow(dead_code)]
      $vis enum $ident {
        Table, $($field),*
      }
      //$(pub use $ident as $alias;)*
    )*
  };
}
