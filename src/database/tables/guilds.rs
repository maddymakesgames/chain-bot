use std::collections::HashMap;

use diesel::{pg::PgConnection, prelude::*, Queryable};
use serenity::model::id::{ChannelId, GuildId};

use crate::{
    bot::guild_settings::GuildSettings,
    database::{schema::*, U64Wrapper},
};

pub fn update_guild(conn: &PgConnection, guild_id: GuildId, settings: &GuildSettings) {
    use self::guilds::dsl::*;

    let row = GuildRow {
        id: guild_id.0.into(),
        prefixes: settings.prefixes.clone(),
        channel_filters: settings
            .channel_filters
            .iter()
            .map(|v| v.0.into())
            .collect(),
        blacklist: settings.blacklist,
        old_style: settings.old_style,
        remove_messages: settings.remove_messages,
        chain_threshold: settings.chain_threshold as i16,
        alternate_member: settings.alternate_member,
    };

    diesel::insert_into(guilds)
        .values(&row)
        .execute(conn)
        .unwrap();
}

pub fn new_guild(conn: &PgConnection, guild_id: GuildId) -> GuildSettings {
    use self::guilds::dsl::*;
    let result: GuildRow = diesel::insert_into(guilds)
        .values(id.eq::<U64Wrapper>(guild_id.0.into()))
        .get_result(conn)
        .unwrap();

    GuildSettings {
        prefixes: result.prefixes,
        channel_filters: result
            .channel_filters
            .iter()
            .map(|v| (*v).into())
            .collect::<Vec<u64>>()
            .iter()
            .map(|v| ChannelId(*v))
            .collect(),
        blacklist: result.blacklist,
        old_style: result.old_style,
        remove_messages: result.remove_messages,
        chain_threshold: result.chain_threshold as u16,
        alternate_member: result.alternate_member,
    }
}

pub fn get_guilds(conn: &PgConnection) -> HashMap<GuildId, GuildSettings> {
    use self::guilds::dsl::*;

    let results = guilds.load::<GuildRow>(conn).unwrap();

    results
        .iter()
        .map(|row| {
            (GuildId(row.id.into()), GuildSettings {
                prefixes: row.prefixes.clone(),
                channel_filters: row
                    .channel_filters
                    .iter()
                    .map(|v| (*v).into())
                    .map(|v: u64| ChannelId(v))
                    .collect(),
                blacklist: row.blacklist,
                old_style: row.old_style,
                remove_messages: row.remove_messages,
                chain_threshold: row.chain_threshold as u16,
                alternate_member: row.alternate_member,
            })
        })
        .collect()
}

#[derive(Insertable, Queryable, Debug)]
#[table_name = "guilds"]
struct GuildRow {
    pub id: U64Wrapper,
    pub prefixes: Vec<String>,
    pub channel_filters: Vec<U64Wrapper>,
    pub blacklist: bool,
    pub old_style: bool,
    pub remove_messages: bool,
    pub chain_threshold: i16,
    pub alternate_member: bool,
}
