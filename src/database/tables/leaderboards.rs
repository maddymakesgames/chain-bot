use diesel::{pg::PgConnection, prelude::*, Queryable};
use serenity::model::id::{GuildId, UserId};

use crate::database::{schema::*, U64Wrapper, UserData};

pub fn get_server_leaderboard_by_points(conn: &PgConnection, guild_id: GuildId) -> Vec<GuildUser> {
    use self::server_users::dsl::*;

    server_users
        .filter(server_id.eq::<U64Wrapper>(guild_id.0.into()))
        .order(points.desc())
        .load::<GuildUser>(conn)
        .unwrap()
}

pub fn get_global_leaderboard_by_points(conn: &PgConnection) -> Vec<UserData> {
    use self::users::dsl::*;

    users.order(points.desc()).load::<UserData>(conn).unwrap()
}

pub fn get_or_create_server_user(
    conn: &PgConnection,
    guild_id: GuildId,
    member_id: UserId,
) -> GuildUser {
    use self::server_users::dsl::*;

    let results = server_users
        .filter(server_id.eq::<U64Wrapper>(guild_id.0.into()))
        .filter(user_id.eq::<U64Wrapper>(member_id.0.into()))
        .load::<GuildUser>(conn)
        .unwrap();

    if results.len() == 0 {
        diesel::insert_into(server_users)
            .values(&GuildUser {
                server_id: guild_id.0.into(),
                user_id: member_id.0.into(),
                points: 0,
                longest_chains: vec![0, 0, 0],
            })
            .load::<GuildUser>(conn)
            .unwrap()
            .first()
            .unwrap()
            .clone()
    } else {
        results.get(0).unwrap().clone()
    }
}

#[derive(Queryable, Clone, Insertable)]
#[table_name = "server_users"]
pub struct GuildUser {
    pub user_id: U64Wrapper,
    pub server_id: U64Wrapper,
    pub points: i64,
    pub longest_chains: Vec<i32>,
}

pub fn increase_server_points(
    conn: &PgConnection,
    guild_id: GuildId,
    member_id: UserId,
    points_to_add: u64,
) {
    use self::server_users::dsl::*;

    diesel::insert_into(server_users)
        .values(&GuildUser {
            server_id: guild_id.0.into(),
            user_id: member_id.0.into(),
            points: points_to_add as i64,
            longest_chains: vec![0, 0, 0],
        })
        .on_conflict((server_id, user_id))
        .do_update()
        .set(points.eq(points + points_to_add as i64))
        .execute(conn)
        .unwrap();
}

pub fn update_server_longest_chains(
    conn: &PgConnection,
    guild_id: GuildId,
    member_id: UserId,
    chain_len: i32,
) {
    use self::server_users::dsl::*;

    let mut user = get_or_create_server_user(conn, guild_id, member_id);

    let hash = user.longest_chains.iter().fold(0, |a, v| a + v);
    println!("{:?}", user.longest_chains);
    user.longest_chains.push(chain_len);
    println!("{:?}", user.longest_chains);
    user.longest_chains.sort();
    user.longest_chains.reverse();
    user.longest_chains.pop();
    println!("{:?}", user.longest_chains);

    if hash != user.longest_chains.iter().fold(0, |a, v| a + v) {
        let filter = server_users.filter(user_id.eq::<U64Wrapper>(member_id.0.into()));
        diesel::update(filter)
            .set(longest_chains.eq(user.longest_chains.clone()))
            .execute(conn)
            .unwrap();
    }
}
