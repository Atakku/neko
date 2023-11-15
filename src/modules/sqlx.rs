// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sea_query::TableCreateStatement;
use sqlx::{postgres::PgPoolOptions, PgPool};

once_cell!(db, POOL: PgPool);

module! {
  Postgres {
    db_url: String = env!("DATABASE_URL"),
    options: PgPoolOptions,
    tables: Vec<TableCreateStatement>
  }

  fn init(fw) {
    rt!(fw, |postgres| {
      POOL.set(postgres.options.connect(&postgres.db_url).await?)?;
      for init in postgres.tables {
        let sql = init.build(sea_query::PostgresQueryBuilder);
        log::info!("{sql}");
        sqlx::query(&sql).execute(db()).await?;
      }
      Ok(None)
    });
  }

  pub fn create_table<T: Creatable>(&mut self) {
    self.tables.push(T::create())
  }

  pub fn create_tables(&mut self, tables: &mut Vec<TableCreateStatement>) {
    self.tables.append(tables)
  }
}

pub trait Creatable {
 fn create() -> TableCreateStatement;
}

/// Declare a new sea_query schema
macro_rules! schema {
  ($(
    $(#[$meta:meta])*
    $vis:vis enum $ident:ident {
      $($field:ident.$($param:ident($($tt:expr),*)).+),*$(,)?
      $(;Self.$($tp:ident($($tpp:expr),*)).+)?
    }
  )*) => {
    $(
      #[derive(sea_query::Iden)]
      $(#[$meta])*
      #[allow(dead_code)]
      $vis enum $ident {
        Table, $($field),*
      }

      impl $crate::modules::sqlx::Creatable for $ident {
        fn create() -> sea_query::TableCreateStatement {
          sea_query::Table::create().table($ident::Table).if_not_exists()
          $(.col(&mut sea_query::ColumnDef::new($ident::$field)$(.$param($($tt),*))*))*
          $($(.$tp($($tpp),*))*)?
          .to_owned()
        }
      }
      //$(pub use $ident as $alias;)*
    )*

    pub fn create_tables() -> Vec<sea_query::TableCreateStatement> {
      use $crate::modules::sqlx::Creatable;
      vec![$($ident::create()),*]
    }
  };
}

macro_rules! fk {
  ($t1:ident, $f1:ident, $t2:ident, $f2:ident, $od:ident, $ou:ident) => {
    sea_query::ForeignKey::create()
      .from($t1::Table, $t1::$f1)
      .to($t2::Table, $t2::$f2)
      .on_delete(sea_query::ForeignKeyAction::$od)
      .on_update(sea_query::ForeignKeyAction::$ou)
  };
  ($t1:ident, $t2:ident, $f:ident, $od:ident, $ou:ident) => {
    fk!($t1, $f, $t2, $f, $od, $ou)
  };
  ($t:ident, $f:ident, $od:ident, $ou:ident) => {
    fk!(Self, $f, $t, $f, $od, $ou)
  };
}

macro_rules! pk {
  ($f1:ident, $f2:ident) => {
    sea_query::Index::create().col(Self::$f1).col(Self::$f2)
  };
}

macro_rules! uk {
  ($f1:ident, $f2:ident) => {
    sea_query::Index::create().unique().col(Self::$f1).col(Self::$f2)
  };
}
