use serenity::{builder::CreateEmbed, model::id::UserId, utils::Colour};
use slashy::{command, commands::CommandResult, framework::CommandContext, subcommand};

use crate::{
    database::{get_or_create_user, tables::leaderboards::get_or_create_server_user},
    DatabaseConn,
};

command! {
    stats,
    "get the stats of a user",
    stats,
    [
        optional User user | "the user who's stats you want to get"
    ]
}


#[subcommand]
async fn stats(ctx: &CommandContext) -> CommandResult {
    let data = ctx.ctx.data.read().await;
    let database = data.get::<DatabaseConn>().unwrap().lock().await;

    let user = get_or_create_user(&database, ctx.author().unwrap().id);
    let self_user = ctx.ctx.http.get_current_user().await?;
    let guild = ctx.guild().await?;
    let server_user = get_or_create_server_user(&database, guild.id, UserId(user.id.into()));
    let caller = ctx.member().await?;
    let member = guild.member(&ctx.ctx, self_user.id).await?;
    let color = member.colour(&ctx.ctx).await.unwrap_or(Colour::MAGENTA);

    drop(database);

    ctx.send_embed(|e: &mut CreateEmbed| {
        e.title(format!("{}'s Stats", caller.display_name()))
            .field("Server Stats", "———————————————", false)
            .field("Points", format!("{} points", server_user.points), true)
            .field(
                "Longest Chains",
                server_user
                    .longest_chains
                    .iter()
                    .enumerate()
                    .fold(String::new(), |mut str, v| {
                        str.push_str(&format!(
                            "{}{}",
                            v.1,
                            if v.0 < user.longest_chains.len() - 1 {
                                ","
                            } else {
                                ""
                            }
                        ));
                        str
                    }),
                true,
            )
            .field("Global Stats", "———————————————", false)
            .field("Points", format!("{} points", user.points), true)
            .field(
                "Longest Chains",
                user.longest_chains
                    .iter()
                    .enumerate()
                    .fold(String::new(), |mut str, v| {
                        str.push_str(&format!(
                            "{}{}",
                            v.1,
                            if v.0 < user.longest_chains.len() - 1 {
                                ","
                            } else {
                                ""
                            }
                        ));
                        str
                    }),
                true,
            )
            .color(color)
    })
    .await?;

    Ok(())
}
