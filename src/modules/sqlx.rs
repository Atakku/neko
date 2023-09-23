// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sqlx::{postgres::PgPoolOptions, PgPool};

once_cell!(db, POOL: PgPool);

module!(
  Postgres {
    db_url: String = expect_env!("DATABASE_URL"),
    options: PgPoolOptions = PgPoolOptions::new(),
  }

  fn init(fw) {
    runtime!(fw, |m| {
      POOL.set(m.options.connect(&m.db_url).await?)?;
      //sqlx::migrate!("./sql").run(db()).await.unwrap();
      Ok(None)
    });
  }
);
