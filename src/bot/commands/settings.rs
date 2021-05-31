use serenity::{
    futures::future::join_all,
    model::{channel::Channel, Permissions},
};
use slashy::{
    command,
    commands::CommandResult,
    framework::CommandContext,
    permissions::ADMINISTRATOR,
    subcommand,
};

use crate::bot::guild_settings::{GuildSettingsStore, DM_SETTINGS};

command! {
    settings,
    "get or set the settings for the server",
    [
        optional SubCommandGroup get = get_settings | "Get settings" [
            optional SubCommand prefixes = get_prefixes | "Get the prefixes",
            optional SubCommand channel_filters = get_filters | "Get the filtered channels",
            optional SubCommand blacklist = get_blacklist | "Get whether we blacklist or whitelist channels in the filter",
            optional SubCommand style = get_style | "Get the response style",
            optional SubCommand remove_messages = get_remove | "Get whether we remove chain messages",
            optional SubCommand chain_threshold = get_threshold | "Get the minimum number of messages required for a chain",
            optional SubCommand alternate = get_alternate | "Get whether you have to alternate to have a valid chain"
        ],
        optional SubCommandGroup set | "Set settings" [
            optional SubCommand prefixes = set_prefix | "Set guild prefixes" [
                required String action | "The action to preform" {"add": "add", "reset": "reset", "remove": "remove"},
                optional String prefix | "The new prefix"
            ],
            optional SubCommand channel_filters = set_filters | "Set the channel filters" [
                required String action | "The action to preform" {"add": "add", "clear": "clear", "remove": "remove"},
                optional Channel channel_id | "The new filter"
            ],
            optional SubCommand blacklist = set_blacklist | "Flip whether we blacklist or whitelist",
            optional SubCommand style = set_style | "Set the style of chain responses" [
                required String style | "The new style" {"embed": "embed", "classic": "classic", "text": "text"}
            ],
            optional SubCommand remove_messages = set_remove | "Flip if we remove messages for chains",
            optional SubCommand chain_threshold = set_threshold | "Set the minimum number of messages to make a chain" [
                required Integer threshold | "The minimum number of messages for a chain"
            ],
            optional SubCommand alternate_messages = set_alternate | "Flip if users need to alternate to make a chain"
        ]
    ]
}

#[subcommand]
async fn get_settings(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.read().await;
    let cache = data.get::<GuildSettingsStore>().unwrap().read().await;
    let settings = cache.get(ctx.guild_id().unwrap()).unwrap();

    if let Ok(Channel::Guild(c)) = ctx.channel().await {
        if c.permissions_for_user(&ctx.ctx, &ctx.ctx.cache.current_user().await.id)
            .await
            .unwrap()
            .contains(Permissions::EMBED_LINKS)
        {
            ctx.send_embed(|e| {
                e.title("Settings");
                e.field("Prefixes", format!("{:?}", settings.prefixes), false);
                e.field(
                    "Channel Filters",
                    format!(
                        "{:?} are {}",
                        settings.channel_filters,
                        if settings.blacklist {
                            "blacklisted"
                        } else {
                            "whitelisted"
                        }
                    ),
                    false,
                );
                e.field("Chain Style", settings.style.clone(), false);
                e.field(
                    "Remove Chain Messages",
                    format!("{}", settings.remove_messages),
                    false,
                );
                e.field(
                    "Chain Threshold",
                    format!("{}", settings.chain_threshold),
                    false,
                );
                e.field(
                    "Alternate Members in Chain",
                    format!("{}", settings.alternate_member),
                    false,
                );

                e
            })
            .await?;
        } else {
            ctx.send_str(&format!(
                r#"```
            Settings for {}
            -------------------------------
            Prefixes: {:?}
            Channel Filters: {:?}
            Filter Type: {}
            Chain Style: {}
            Remove Chain Messages: {}
            Chain Threshold: {}
            Alternate Members: {}
            ```"#,
                ctx.guild().await?.name,
                settings.prefixes,
                settings.channel_filters,
                if settings.blacklist {
                    "blacklist"
                } else {
                    "whitelist"
                },
                settings.style,
                settings.remove_messages,
                settings.chain_threshold,
                settings.alternate_member
            ))
            .await?;
        }
    }

    Ok(())
}

#[subcommand]
async fn get_prefixes(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.read().await;
    let settings = data.get::<GuildSettingsStore>().unwrap().read().await;
    let prefixes = settings.prefixes(ctx.guild_id().unwrap());

    ctx.send_str(&format!("The server prefixes are {:?}", prefixes))
        .await?;

    Ok(())
}

#[subcommand]
async fn get_filters(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.read().await;
    let settings = data.get::<GuildSettingsStore>().unwrap().read().await;
    let filters = settings
        .channel_filters(ctx.guild_id().unwrap())
        .iter()
        .map(|c| c.name(&ctx.ctx))
        .collect::<Vec<_>>();

    let filters = join_all(filters).await;
    let filters = filters.iter().flatten().collect::<Vec<_>>();

    ctx.send_str(&format!("The channel filter is {:?}", filters))
        .await?;

    Ok(())
}

#[subcommand]
async fn get_blacklist(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.read().await;
    let settings = data.get::<GuildSettingsStore>().unwrap().read().await;
    let blacklist = settings.blacklist(ctx.guild_id().unwrap());
    ctx.send_str(&format!(
        "The channel filter acts as a {}",
        if blacklist { "blacklist" } else { "whitelist" }
    ))
    .await?;

    Ok(())
}

#[subcommand]
async fn get_style(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.read().await;
    let settings = data.get::<GuildSettingsStore>().unwrap().read().await;
    let style = settings.style(ctx.guild_id().unwrap());
    ctx.send_str(&format!("The chain style is {}", style))
        .await?;

    Ok(())
}

#[subcommand]
async fn get_remove(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.read().await;
    let settings = data.get::<GuildSettingsStore>().unwrap().read().await;
    let delete = settings.remove_messages(ctx.guild_id().unwrap());
    ctx.send_str(&format!(
        "Messages in a chain are{} deleted",
        if delete { "" } else { " not" }
    ))
    .await?;

    Ok(())
}

#[subcommand]
async fn get_threshold(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.read().await;
    let settings = data.get::<GuildSettingsStore>().unwrap().read().await;
    let threshold = settings.chain_threshold(ctx.guild_id().unwrap());
    ctx.send_str(&format!(
        "Members need {} messages to form a chain",
        threshold
    ))
    .await?;

    Ok(())
}

#[subcommand]
async fn get_alternate(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.read().await;
    let settings = data.get::<GuildSettingsStore>().unwrap().read().await;
    let alternate = settings.alternate_member(ctx.guild_id().unwrap());
    ctx.send_str(&format!(
        "Members{} need to alternate in order to make a chain",
        if alternate { "" } else { " do not" }
    ))
    .await?;

    Ok(())
}

// Arguments:
// Optional String prefix
// String action
#[subcommand(ADMINISTRATOR)]
async fn set_prefix(ctx: &CommandContext) -> CommandResult {
    let action = ctx.get_str_arg("action").unwrap();

    match action.as_str() {
        "add" =>
            if ctx.get_str_arg("prefix").is_some() {
                add_prefix(ctx).await?;
            } else {
                ctx.send_str("You need to provide a prefix to add").await?;
            },
        "reset" => {
            reset_prefix(ctx).await?;
        }
        "remove" =>
            if ctx.get_str_arg("prefix").is_some() {
                remove_prefix(ctx).await?;
            } else {
                ctx.send_str("You need to provide a prefix to remove")
                    .await?;
            },
        _ => unreachable!(),
    }

    Ok(())
}


async fn add_prefix(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.write().await;
    let mut settings = data.get::<GuildSettingsStore>().unwrap().write().await;
    let guild_id = ctx.guild_id().unwrap();

    (*settings.prefixes_mut(guild_id)).push(ctx.get_str_arg("prefix").unwrap().clone());

    ctx.send_str(&format!(
        "Prefix {} added",
        ctx.get_str_arg("prefix").unwrap()
    ))
    .await?;

    settings.save_guild(guild_id);

    Ok(())
}

async fn reset_prefix(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.write().await;
    let mut settings = data.get::<GuildSettingsStore>().unwrap().write().await;
    let guild_id = ctx.guild_id().unwrap();

    *settings.prefixes_mut(guild_id) = DM_SETTINGS.prefixes.clone();

    ctx.send_str("Prefixes reset").await?;

    settings.save_guild(guild_id);

    Ok(())
}

async fn remove_prefix(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.write().await;
    let mut settings = data.get::<GuildSettingsStore>().unwrap().write().await;
    let guild_id = ctx.guild_id().unwrap();

    let prefixes = settings.prefixes_mut(guild_id);
    let removal = ctx.get_str_arg("prefix").unwrap();

    if prefixes.len() == 1 {
        ctx.send_str("you can't remove a prefix if you only have one")
            .await?;
    } else {
        *prefixes = prefixes
            .iter()
            .filter(|p| *p != removal)
            .map(|p| p.clone())
            .collect::<Vec<String>>();

        ctx.send_str(&format!("Prefix {} removed", removal)).await?;

        settings.save_guild(guild_id);
    }

    Ok(())
}

// Arguments:
// Channel channel_id
#[subcommand(ADMINISTRATOR)]
async fn set_filters(ctx: &CommandContext) -> CommandResult {
    let action = ctx.get_str_arg("action").unwrap();

    match action.as_str() {
        "add" =>
            if ctx.get_str_arg("channel_id").is_some() {
                add_filter(ctx).await?;
            } else {
                ctx.send_str("You need to provide a channel id to add")
                    .await?;
            },
        "clear" => {
            clear_filters(ctx).await?;
        }
        "remove" =>
            if ctx.get_str_arg("channel_id").is_some() {
                remove_filter(ctx).await?;
            } else {
                ctx.send_str("You need to provide a channel id to remove")
                    .await?;
            },
        _ => unreachable!(),
    }

    Ok(())
}


async fn add_filter(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.write().await;
    let mut settings = data.get::<GuildSettingsStore>().unwrap().write().await;
    let guild_id = ctx.guild_id().unwrap();

    (*settings.channel_filters_mut(guild_id))
        .push(ctx.get_channel_arg("channel_id").unwrap().clone());

    ctx.send_str("Added new channel filter").await?;

    settings.save_guild(guild_id);

    Ok(())
}

async fn clear_filters(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.write().await;
    let mut settings = data.get::<GuildSettingsStore>().unwrap().write().await;
    let guild_id = ctx.guild_id().unwrap();

    *settings.channel_filters_mut(guild_id) = Vec::new();

    ctx.send_str("Cleared channel filters").await?;

    settings.save_guild(guild_id);

    Ok(())
}

async fn remove_filter(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.write().await;
    let mut settings = data.get::<GuildSettingsStore>().unwrap().write().await;
    let guild_id = ctx.guild_id().unwrap();

    let channels = settings.channel_filters_mut(guild_id);
    let removal = ctx.get_channel_arg("channel_id").unwrap();

    *channels = channels
        .iter()
        .filter(|p| *p != removal)
        .map(|p| *p)
        .collect::<Vec<_>>();

    ctx.send_str("Removed channel filter").await?;

    settings.save_guild(guild_id);

    Ok(())
}


#[subcommand(ADMINISTRATOR)]
async fn set_blacklist(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.write().await;
    let mut settings = data.get::<GuildSettingsStore>().unwrap().write().await;
    let guild_id = ctx.guild_id().unwrap();

    *settings.blacklist_mut(guild_id) ^= true;

    ctx.send_str("Flipped if we are blacklisting or whitelisting")
        .await?;

    settings.save_guild(guild_id);

    Ok(())
}

#[subcommand(ADMINISTRATOR)]
async fn set_remove(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.write().await;
    let mut settings = data.get::<GuildSettingsStore>().unwrap().write().await;
    let guild_id = ctx.guild_id().unwrap();

    *settings.remove_messages_mut(guild_id) ^= true;

    ctx.send_str("Flipped if we remove chain messages").await?;

    settings.save_guild(guild_id);

    Ok(())
}

// Arguments: Int threshold
#[subcommand(ADMINISTRATOR)]
async fn set_threshold(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.write().await;
    let mut settings = data.get::<GuildSettingsStore>().unwrap().write().await;
    let guild_id = ctx.guild_id().unwrap();

    *settings.chain_threshold_mut(guild_id) = ctx.get_int_arg("threshold").unwrap().clone() as u16;

    ctx.send_str(&format!(
        "Set the threshold to {}",
        ctx.get_int_arg("threshold").unwrap()
    ))
    .await?;

    settings.save_guild(guild_id);

    Ok(())
}

#[subcommand(ADMINISTRATOR)]
async fn set_alternate(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.write().await;
    let mut settings = data.get::<GuildSettingsStore>().unwrap().write().await;
    let guild_id = ctx.guild_id().unwrap();

    *settings.alternate_member_mut(guild_id) ^= true;

    ctx.send_str("Flipped if members need to alternate to chain")
        .await?;

    settings.save_guild(guild_id);

    Ok(())
}

// Arguments: String style
#[subcommand(ADMINISTRATOR)]
async fn set_style(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.write().await;
    let mut settings = data.get::<GuildSettingsStore>().unwrap().write().await;
    let guild_id = ctx.guild_id().unwrap();

    *settings.style_mut(guild_id) = ctx.get_str_arg("style").unwrap().clone();

    ctx.send_str(&format!(
        "Set the chain style to {}",
        ctx.get_str_arg("style").unwrap()
    ))
    .await?;

    settings.save_guild(guild_id);

    Ok(())
}
