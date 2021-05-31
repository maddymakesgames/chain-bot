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
        style: settings.style.clone(),
        remove_messages: settings.remove_messages,
        chain_threshold: settings.chain_threshold as i16,
        alternate_member: settings.alternate_member,
    };

    diesel::update(guilds.filter(id.eq::<U64Wrapper>(guild_id.0.into())))
        .set((
            prefixes.eq(row.prefixes),
            channel_filters.eq(row.channel_filters),
            blacklist.eq(row.blacklist),
            style.eq(row.style),
            remove_messages.eq(row.remove_messages),
            chain_threshold.eq(row.chain_threshold),
            alternate_member.eq(row.alternate_member),
        ))
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
        style: result.style,
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
                style: row.style.clone(),
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
    pub style: String,
    pub remove_messages: bool,
    pub chain_threshold: i16,
    pub alternate_member: bool,
}

macro_rules! update_setting {
    ($($method_name: ident, $param: ident, $field: ident, $type: ty),*) => {
        $(pub fn $method_name(conn: &PgConnection, guild_id: GuildId, $param: $type) {
            use self::guilds::dsl::*;

            diesel::update(guilds.filter(id.eq::<U64Wrapper>(guild_id.0.into())))
                .set($field.eq($param))
                .execute(conn)
                .unwrap();
        })*
    };
}

update_setting!(
    update_prefix,
    new_prefixes,
    prefixes,
    Vec<String>,
    update_filters,
    new_filters,
    channel_filters,
    Vec<U64Wrapper>,
    update_blacklist,
    blacklist_flag,
    blacklist,
    bool,
    update_style,
    new_style,
    style,
    String,
    update_remove,
    remove_flag,
    remove_messages,
    bool,
    update_threshold,
    new_threshold,
    chain_threshold,
    i16,
    update_alternate,
    alternate_flag,
    alternate_member,
    bool
);
