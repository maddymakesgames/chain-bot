use std::{collections::HashMap, sync::Arc};

use diesel::PgConnection;
use serenity::{
    model::id::{ChannelId, GuildId},
    prelude::{Mutex, RwLock, TypeMapKey},
    FutureExt,
};

use lazy_static::lazy_static;
use slashy::settings::SettingsProvider;

use crate::database::{establish_connection, get_guilds, new_guild, update_guild};

pub struct GuildSettingsStore;

impl TypeMapKey for GuildSettingsStore {
    type Value = Arc<RwLock<GuildSettingsCache>>;
}

pub struct GuildSettingsCache {
    guild_map: HashMap<GuildId, GuildSettings>,
    database_connection: Mutex<PgConnection>,
    testing_guilds: Vec<GuildId>,
}

macro_rules! get_field {
    ($field:ident, $field_mut:ident, $type: ty) => {
        pub async fn $field<'a>(&'a mut self, guild_id: GuildId) -> &'a $type {
            &self.get_or_default(guild_id).await.$field
        }

        pub async fn $field_mut<'a>(&'a mut self, guild_id: GuildId) -> &'a mut $type {
            &mut self.get_mut_or_default(guild_id).await.$field
        }
    };
}
#[allow(dead_code)]
impl GuildSettingsCache {
    get_field! {prefixes, prefixes_mut, Vec<String>}

    get_field! {channel_filters, channel_filters_mut, Vec<ChannelId>}

    get_field! {blacklist, blacklist_mut, bool}

    get_field! {old_style, old_style_mut, bool}

    get_field! {remove_messages, remove_messages_mut, bool}

    get_field! {chain_threshold, chain_threshold_mut, u16}

    get_field! {alternate_member, alternate_member_mut, bool}

    pub fn new(testing_guilds: Vec<GuildId>) -> Self {
        GuildSettingsCache {
            database_connection: Mutex::new(establish_connection()),
            guild_map: HashMap::new(),
            testing_guilds,
        }
    }

    async fn save(&self) {
        let conn = self.database_connection.lock().await;
        for (id, settings) in &self.guild_map {
            update_guild(&conn, *id, settings)
        }
    }

    pub async fn load_guilds(&mut self) {
        let conn = self.database_connection.lock().await;
        self.guild_map = get_guilds(&conn);
    }

    pub fn get<'a>(&'a self, guild_id: GuildId) -> Option<&'a GuildSettings> {
        self.guild_map.get(&guild_id)
    }

    pub fn get_mut<'a>(&'a mut self, guild_id: GuildId) -> Option<&'a mut GuildSettings> {
        self.guild_map.get_mut(&guild_id)
    }

    pub async fn get_or_default<'a>(&'a mut self, guild_id: GuildId) -> &'a GuildSettings {
        if self.guild_map.contains_key(&guild_id) {
            self.guild_map.get(&guild_id).unwrap()
        } else {
            let conn = self.database_connection.lock().await;
            let guild_settings = new_guild(&conn, guild_id);
            self.guild_map.insert(guild_id, guild_settings);
            self.guild_map.get(&guild_id).unwrap()
        }
    }

    pub async fn get_mut_or_default<'a>(&'a mut self, guild_id: GuildId) -> &'a mut GuildSettings {
        if self.guild_map.contains_key(&guild_id) {
            self.guild_map.get_mut(&guild_id).unwrap()
        } else {
            let conn = self.database_connection.lock().await;
            let guild_settings = new_guild(&conn, guild_id);
            self.guild_map.insert(guild_id, guild_settings);
            self.guild_map.get_mut(&guild_id).unwrap()
        }
    }
}

impl Drop for GuildSettingsCache {
    // Save guild settings before we drop to make sure nothing has gone wrong
    fn drop(&mut self) {
        self.save().boxed().now_or_never();
    }
}

impl SettingsProvider for GuildSettingsCache {
    fn default_prefixes(&self) -> Vec<String> {
        DM_SETTINGS.prefixes.clone()
    }

    fn prefixes(&self, guild_id: GuildId) -> Option<Vec<String>> {
        self.get(guild_id).map(|g| g.prefixes.clone())
    }

    fn auto_register(&self) -> bool {
        true
    }

    fn auto_delete(&self) -> bool {
        true
    }

    fn auto_register_guilds(&self) -> Vec<GuildId> {
        self.testing_guilds.clone()
    }
}

#[derive(Clone)]
pub struct GuildSettings {
    pub prefixes: Vec<String>,
    pub channel_filters: Vec<ChannelId>,
    pub blacklist: bool,
    pub old_style: bool,
    pub remove_messages: bool,
    pub chain_threshold: u16,
    pub alternate_member: bool,
}

lazy_static! {
    pub static ref DM_SETTINGS: GuildSettings = GuildSettings {
        prefixes: vec!["cb.".to_owned()],
        channel_filters: Vec::new(),
        blacklist: false,
        old_style: false,
        remove_messages: true,
        chain_threshold: u16::max_value(),
        alternate_member: true
    };
}
