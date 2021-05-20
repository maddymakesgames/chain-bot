use diesel::{pg::PgConnection, prelude::*, Queryable};
use serenity::model::id::UserId;

use crate::database::{schema::*, U64Wrapper};

pub fn create_user(conn: &PgConnection, id: UserId) -> UserData {
    diesel::insert_into(users::table)
        .values(&UserData {
            id: id.0.into(),
            points: 0,
            longest_chains: vec![0, 0, 0],
        })
        .get_result(conn)
        .expect("Error creating new user")
}

pub fn get_or_create_user(conn: &PgConnection, user_id: UserId) -> UserData {
    use self::users::dsl::*;

    let results = users
        .filter(id.eq::<U64Wrapper>(user_id.0.into()))
        .limit(1)
        .load::<UserData>(conn)
        .expect("Error getting user data");

    if results.len() == 0 {
        diesel::insert_into(users)
            .values(&UserData {
                id: user_id.0.into(),
                points: 0,
                longest_chains: vec![0, 0, 0],
            })
            .load::<UserData>(conn)
            .unwrap()
            .get(0)
            .unwrap()
            .clone()
    } else {
        results.get(0).unwrap().clone()
    }
}

pub fn increase_points(conn: &PgConnection, user_id: UserId, points_to_add: u64) {
    use self::users::dsl::*;

    diesel::insert_into(users)
        .values(&UserData {
            id: user_id.0.into(),
            points: points_to_add as i64,
            longest_chains: vec![0, 0, 0],
        })
        .on_conflict(id)
        .do_update()
        .set(points.eq(points + points_to_add as i64))
        .execute(conn)
        .unwrap();
}

pub fn update_longest_chains(conn: &PgConnection, user_id: UserId, chain_len: i32) {
    use self::users::dsl::*;

    let mut user = get_or_create_user(conn, user_id);

    let hash = user.longest_chains.iter().fold(0, |a, v| a + v);
    println!("{:?}", user.longest_chains);
    user.longest_chains.push(chain_len);
    println!("{:?}", user.longest_chains);
    user.longest_chains.sort();
    user.longest_chains.reverse();
    user.longest_chains.pop();
    println!("{:?}", user.longest_chains);

    if hash != user.longest_chains.iter().fold(0, |a, v| a + v) {
        let filter = users.filter(id.eq::<U64Wrapper>(user_id.0.into()));
        diesel::update(filter)
            .set(longest_chains.eq(user.longest_chains.clone()))
            .execute(conn)
            .unwrap();
    }
}

#[derive(Queryable, Clone, Insertable)]
#[table_name = "users"]
pub struct UserData {
    pub id: U64Wrapper,
    pub points: i64,
    pub longest_chains: Vec<i32>,
}
