use std::{borrow::Cow, cmp::min, collections::HashMap};

use serenity::{
    async_trait,
    client::{Context, EventHandler},
    futures::future::{join_all, try_join_all},
    model::{
        channel::{Channel, GuildChannel, Message},
        guild::Member,
        id::{ChannelId, GuildId, MessageId, UserId},
    },
    prelude::{TypeMap, TypeMapKey},
    utils::Color,
};
use tokio::join;

use crate::{
    bot::guild_settings::{GuildSettings, GuildSettingsStore},
    database::{tables::leaderboards::update_server_longest_chains, update_longest_chains},
    DatabaseConn,
};

use super::points::{give_points, points_per_user};

pub struct ChainCounter;

pub type ChainStore = HashMap<ChannelId, Chain>;

#[derive(Clone)]
pub struct Chain {
    pub message: String,
    pub msg_cache: Vec<Message>,
    pub chainers: Vec<UserId>,
    pub num_messages: HashMap<UserId, u16>,
    pub starter: Member,
    pub length: u16,
}

impl TypeMapKey for ChainCounter {
    type Value = ChainStore;
}
pub struct ChainHandler;

#[async_trait]
impl EventHandler for ChainHandler {
    async fn message(&self, ctx: Context, message: Message) {
        if message.author.bot || message.guild_id.is_none() {
            return;
        }

        // Get the chain cache
        let mut data = ctx.data.write().await;

        let chains = data
        .get_mut::<ChainCounter>()
        // If we can't get this something has gone horribly wrong and a panic is justified
            .expect("Error getting ChainCounter from Context");

        // Store the ids we use a lot
        let author_id = message.author.id;
        let channel_id = message.channel_id;

        if !chains.contains_key(&channel_id) {
            // If we do not already have a chain in that channel, make a new chain
            create_chain(&message, &ctx, chains).await;
        } else if chains.get(&channel_id).unwrap().message == message.content {
            // If we are continuing the chain, update it and write the changes
            let chain = chains.get_mut(&channel_id).unwrap();
            chain.length += 1;
            chain.msg_cache.push(message.clone());

            // Set or increment the chainer's number of messages
            if !chain.chainers.contains(&author_id) {
                chain.chainers.push(author_id);
                chain.num_messages.insert(author_id, 1);
            } else {
                *chain.num_messages.get_mut(&author_id).unwrap() += 1
            }
        } else {
            // If we are breaking the chain, get the points and cleanup the chain
            let chain = chains.get(&channel_id).unwrap().clone();

            let points = points_per_user(&chain, author_id);

            // Then remove the chain from the cache
            let chain = chain.clone();
            chains.remove(&channel_id);

            // Manually drop chains as we need data for the final step
            drop(chains);

            let settings_store = data
                .get::<GuildSettingsStore>()
                .expect("Error getting GuildSettingsStore")
                .read()
                .await;

            let guild_settings = settings_store.get(message.guild_id.unwrap()).unwrap();

            // And update points and user info
            join!(
                give_points(&points, &data, message.guild_id.unwrap()),
                update_chain_data(&chain, &data, message.guild_id.unwrap()),
                cleanup_chain(&chain, &message, &ctx),
                create_chain_response(&chain, &points, &message, &ctx, &guild_settings)
            );
        }
    }
}

async fn create_chain(message: &Message, ctx: &Context, chains: &mut ChainStore) {
    let channel_id = message.channel_id;
    let author_id = message.author.id;

    let channel = message
        .channel_id
        .to_channel(&ctx)
        .await
        .expect("Error getting channel from message");

    match channel {
        Channel::Guild(c) => {
            let messages = c
                .messages(&ctx, |b| b.limit(2))
                .await
                .expect("Error getting messages");
            let msg = messages.get(1).unwrap();
            if message.content == msg.content && !msg.author.bot {
                let mut num_messages = HashMap::new();
                if msg.author.id == author_id {
                    num_messages.insert(author_id, 2);
                } else {
                    num_messages.insert(author_id, 1);
                    num_messages.insert(msg.author.id, 1);
                }

                let guild = message.guild(&ctx).await.unwrap();

                chains.insert(channel_id, Chain {
                    message: message.content.clone(),
                    msg_cache: vec![msg.clone(), message.clone()],
                    chainers: if msg.author.id == author_id {
                        vec![msg.author.id]
                    } else {
                        vec![msg.author.id, author_id]
                    },
                    num_messages,
                    starter: guild
                        .member(&ctx, author_id)
                        .await
                        .expect("Error getting message"),
                    length: 2,
                });
            }
        }
        _ => {}
    }
}

async fn create_chain_response(
    chain: &Chain,
    points: &HashMap<UserId, u64>,
    message: &Message,
    ctx: &Context,
    settings: &GuildSettings,
) {
    if chain.length > 5 {
        let guild = message.guild(&ctx).await.unwrap();
        let breaker = message
            .member
            .clone()
            .unwrap()
            .nick
            .unwrap_or(message.author.name.clone());

        let user = ctx.http.get_current_user().await.unwrap().id;
        let member = guild.member(ctx, user).await.unwrap();
        let color = member
            .colour(ctx)
            .await
            .unwrap_or(Color::from_rgb(120, 5, 90));

        let mut members = Vec::new();

        for m_id in &chain.chainers {
            members.push(guild.member(&ctx, m_id));
        }

        let mut members = join_all(members)
            .await
            .into_iter()
            .filter_map(|m| m.map(|m| Some(m)).unwrap_or(None))
            .collect::<Vec<_>>();

        if !chain.chainers.contains(&message.author.id) {
            members.push(message.member(&ctx).await.unwrap());
        }

        message
            .channel_id
            .send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title(format!("{} chain!", chain.length));
                    e.description(format!(
                        "{} made a chain of {}",
                        members
                            .iter()
                            .map(|m| m.display_name())
                            .collect::<Vec<Cow<String>>>()
                            .iter()
                            .enumerate()
                            .fold(String::new(), |mut acc, (i, v)| {
                                acc.push_str(&format!(
                                    "{}{}",
                                    v,
                                    if i == members.len() - 1 { "" } else { ", " }
                                ));
                                acc
                            }),
                        chain.length
                    ));
                    e.color(color);
                    e.field(
                        "starter",
                        chain
                            .starter
                            .nick
                            .clone()
                            .unwrap_or(chain.starter.user.name.clone()),
                        true,
                    );
                    e.field("breaker", breaker, true);
                    e.field(
                        "points",
                        points
                            .iter()
                            .map(|(id, p)| {
                                let member =
                                    members.iter().filter(|m| &m.user.id == id).next().unwrap();
                                (member.nick.clone().unwrap_or(member.user.name.clone()), p)
                            })
                            .enumerate()
                            .fold(String::new(), |mut str, (i, (m, p))| {
                                str.push_str(&format!(
                                    "{}: {} points{}",
                                    m,
                                    p,
                                    if i == points.len() - 1 { "" } else { "\n" }
                                ));
                                str
                            }),
                        false,
                    );
                    e
                });
                m
            })
            .await
            .unwrap();
    }
}

async fn cleanup_chain(chain: &Chain, message: &Message, ctx: &Context) {
    println!("cleanup_chain {}", chain.length);
    if chain.length < 5 {
        return;
    }

    if let Channel::Guild(c) = message.channel(&ctx).await.unwrap() {
        let mut user_map = Vec::new();
        let mut ids = Vec::new();

        for msg in &chain.msg_cache {
            let author = msg.author.id;
            if !user_map.contains(&author) {
                user_map.push(author);
            } else {
                ids.push(msg.id);
            }
        }

        println!("Deleting {:?} from users {:?}", ids.len(), user_map);

        mass_delete(ctx, ids, c).await;
    }
}

async fn update_chain_data(chain: &Chain, data: &TypeMap, guild_id: GuildId) {
    let database = data.get::<DatabaseConn>().unwrap().lock().await;

    for user in &chain.chainers {
        update_longest_chains(&database, *user, chain.length as i32);
        update_server_longest_chains(&database, guild_id, *user, chain.length as i32);
    }
}


/// Bulk delete any number of messages over 2
async fn mass_delete(ctx: &Context, messages: Vec<MessageId>, channel: GuildChannel) {
    if messages.len() <= 100 {
        channel.delete_messages(&ctx, messages).await.unwrap()
    } else {
        let mut messages = messages.split_at(100);
        let mut futures = Vec::new();

        while messages.1.len() != 0 {
            futures.push(channel.delete_messages(&ctx, messages.0));

            messages = messages.1.split_at(min(messages.1.len(), 100))
        }

        try_join_all(futures).await.unwrap();
    }
}
