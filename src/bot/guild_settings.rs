use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use diesel::PgConnection;
use serenity::{
    model::id::{ChannelId, GuildId},
    prelude::{RwLock, TypeMapKey},
};

use lazy_static::lazy_static;
use slashy::settings::SettingsProvider;

use crate::database::{establish_connection, guilds::*};

pub struct GuildSettingsStore;

impl TypeMapKey for GuildSettingsStore {
    type Value = Arc<RwLock<GuildSettingsCache>>;
}

pub struct GuildSettingsCache {
    guild_map: HashMap<GuildId, GuildSettings>,
    // GuildSettingsCache has its own PgConnection unrelated to the main connection used in commands
    // While this does mean we could attempt to write to the database in multiple places, somewhat invalidating the Mutex
    // PGSQL will handle multiple writes for us and we should always be writing to different tables so it doesn't matter
    // This mutex is mainly used to allow PgConnection to be sent over threads
    database_connection: Arc<Mutex<PgConnection>>,
    testing_guilds: Vec<GuildId>,
}

macro_rules! get_field {
    ($($field:ident, $field_mut:ident, $type: ty),*) => {
        $(pub fn $field(&self, guild_id: GuildId) -> $type {
            self.get_or_default(guild_id).$field
        }

        pub fn $field_mut(&mut self, guild_id: GuildId) -> &mut $type {
            &mut self.get_mut_or_default(guild_id).$field
        })*
    };
}
#[allow(dead_code)]
impl GuildSettingsCache {
    get_field! {
        prefixes, prefixes_mut, Vec<String>,
        channel_filters, channel_filters_mut, Vec<ChannelId>,
        blacklist, blacklist_mut, bool,
        style, style_mut, String,
        remove_messages, remove_messages_mut, bool,
        chain_threshold, chain_threshold_mut, u16,
        alternate_member, alternate_member_mut, bool
    }

    pub fn new(testing_guilds: Vec<GuildId>) -> Self {
        GuildSettingsCache {
            database_connection: Arc::new(Mutex::new(establish_connection())),
            guild_map: HashMap::new(),
            testing_guilds,
        }
    }

    pub fn save(&self) {
        let conn = self.database_connection.lock().unwrap();
        for (id, settings) in &self.guild_map {
            update_guild(&conn, *id, settings)
        }
    }

    pub fn save_guild(&self, guild_id: GuildId) {
        let conn = self.database_connection.lock().unwrap();
        let guild = self.get_or_default(guild_id);
        update_guild(&conn, guild_id, &guild)
    }

    pub fn load_guilds(&mut self) {
        let conn = self.database_connection.lock().unwrap();
        self.guild_map = get_guilds(&conn);
    }

    pub fn get(&self, guild_id: GuildId) -> Option<&GuildSettings> {
        self.guild_map.get(&guild_id)
    }

    pub fn get_mut<'a>(&'a mut self, guild_id: GuildId) -> Option<&mut GuildSettings> {
        self.guild_map.get_mut(&guild_id)
    }

    pub fn get_or_default(&self, guild_id: GuildId) -> GuildSettings {
        if self.guild_map.contains_key(&guild_id) {
            self.guild_map.get(&guild_id).unwrap().clone()
        } else {
            DM_SETTINGS.to_owned()
        }
    }

    pub fn get_mut_or_default(&mut self, guild_id: GuildId) -> &mut GuildSettings {
        if self.guild_map.contains_key(&guild_id) {
            self.guild_map.get_mut(&guild_id).unwrap()
        } else {
            let conn = self.database_connection.lock().unwrap();
            let guild_settings = new_guild(&conn, guild_id);
            self.guild_map.insert(guild_id, guild_settings);
            self.guild_map.get_mut(&guild_id).unwrap()
        }
    }
}

impl Drop for GuildSettingsCache {
    // Save guild settings before we drop to make sure nothing has gone wrong
    fn drop(&mut self) {
        self.save();
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
    pub style: String,
    pub remove_messages: bool,
    pub chain_threshold: u16,
    pub alternate_member: bool,
}

lazy_static! {
    pub static ref DM_SETTINGS: GuildSettings = GuildSettings {
        prefixes: vec!["cb.".to_owned()],
        channel_filters: Vec::new(),
        blacklist: false,
        style: "embed".to_owned(),
        remove_messages: true,
        chain_threshold: u16::max_value(),
        alternate_member: true
    };
}
