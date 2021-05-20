use std::io::Write;

use diesel::{
    backend::Backend,
    deserialize::Queryable,
    serialize::Output,
    sql_types::BigInt,
    types::{FromSql, ToSql},
    Connection,
    PgConnection,
};

pub use super::tables::{guilds::*, users::*};

pub fn establish_connection() -> PgConnection {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", &database_url))
}

#[derive(Debug, PartialEq, Eq, AsExpression, Clone, Copy)]
#[sql_type = "BigInt"]
pub struct U64Wrapper(pub u64);

impl From<u64> for U64Wrapper {
    fn from(c: u64) -> Self {
        U64Wrapper(c)
    }
}

impl Into<u64> for U64Wrapper {
    fn into(self) -> u64 {
        self.0
    }
}

impl<DB: Backend> FromSql<BigInt, DB> for U64Wrapper
where i64: FromSql<BigInt, DB>
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
        Ok((i64::from_sql(bytes)? as u64).into())
    }
}

impl<DB: Backend> ToSql<BigInt, DB> for U64Wrapper
where i64: ToSql<BigInt, DB>
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> diesel::serialize::Result {
        (self.0 as i64).to_sql(out)
    }
}

impl<DB: Backend> Queryable<BigInt, DB> for U64Wrapper
where i64: Queryable<BigInt, DB>
{
    type Row = <i64 as Queryable<BigInt, DB>>::Row;

    fn build(row: Self::Row) -> Self {
        (i64::build(row) as u64).into()
    }
}
