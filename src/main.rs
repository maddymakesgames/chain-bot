#[macro_use]
extern crate diesel;

use std::{collections::HashMap, sync::Arc};

use bot::guild_settings::{GuildSettingsCache, GuildSettingsStore};
use chain::{ChainCounter, ChainHandler};
use diesel::PgConnection;

use serenity::{
    futures::lock::Mutex,
    model::id::GuildId,
    prelude::{RwLock, TypeMapKey},
    Client,
};

use slashy::framework::Framework;

use bot::commands::*;


mod bot;
mod chain;
mod database;
// pub mod interactions;

pub struct DatabaseConn;
impl TypeMapKey for DatabaseConn {
    type Value = Arc<Mutex<PgConnection>>;
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let connection = database::establish_connection();

    let token = std::env::var("DISCORD_TOKEN").expect("No token provided");
    let application_id = std::env::var("APPLICATION_ID").expect("No application id provided");
    let application_id = serde_json::from_str::<u64>(&application_id).unwrap();

    let testing_guilds = std::env::var("TESTING_GUILDS").unwrap_or("[]".to_owned());
    let testing_guilds =
        serde_json::from_str::<Vec<GuildId>>(&testing_guilds).expect("Error in TESTING_GUILDS");

    let guild_setting_cache = Arc::new(RwLock::new(GuildSettingsCache::new(testing_guilds)));

    let framework = Framework::new(guild_setting_cache.clone(), application_id, token.clone())
        .await
        .event_handler(ChainHandler)
        .command::<TOP_COMMAND>()
        .command::<STATS_COMMAND>();


    let mut client = Client::builder(token)
        .application_id(application_id)
        .event_handler(framework)
        .await
        .expect("Error making client");

    let mut data = client.data.write().await;
    data.insert::<ChainCounter>(HashMap::default());
    data.insert::<DatabaseConn>(Arc::new(Mutex::new(connection)));

    guild_setting_cache.write().await.load_guilds().await;
    data.insert::<GuildSettingsStore>(guild_setting_cache);

    drop(data);

    if let Err(err) = client.start().await {
        println!("Client encountered an error: {:?}", err);
    }
}
