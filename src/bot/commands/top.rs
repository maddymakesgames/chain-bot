use serenity::futures::future::join_all;
use slashy::{
    argument::Argument,
    command,
    commands::CommandResult,
    framework::CommandContext,
    subcommand,
};

use crate::{
    database::tables::leaderboards::{
        get_global_leaderboard_by_points,
        get_server_leaderboard_by_points,
    },
    DatabaseConn,
};

command! {
    top,
    "get the points leaderboard for either the server or globally",
    top,
    [
        optional Integer page | "the page of the leaderboard",
        optional Boolean global | "whether to use the global leaderboard"
    ]
}

#[subcommand]
async fn top(ctx: &CommandContext) -> CommandResult {
    // Get arguments
    let start_index = match ctx.get_arg("page") {
        Some(Argument::Integer(i)) => i * 10,
        _ => 0,
    };

    // Get the database connection
    let data = ctx.ctx.data.read().await;
    let database = data.get::<DatabaseConn>().unwrap().lock().await;


    let rankings = match ctx.guild_id() {
        Some(g) => {
            let leaderboard = get_server_leaderboard_by_points(&database, g);

            let leaderboard_slice = if start_index >= 0
                && (start_index as usize) < leaderboard.len() - 1
                && start_index as usize + 10 < leaderboard.len() - 1
            {
                &leaderboard[start_index as usize .. start_index as usize + 10]
            } else if start_index >= 0 && (start_index as usize) < leaderboard.len() - 1 {
                &leaderboard[start_index as usize ..]
            } else {
                &[]
            };


            let mut futures = Vec::new();

            for (placement, user) in leaderboard_slice.iter().enumerate() {
                futures.push(async move {
                    format!(
                        "{}: {} ({} points)",
                        placement as i32 + start_index + 1,
                        g.member(&ctx.ctx, user.user_id.0)
                            .await
                            .unwrap()
                            .display_name(),
                        user.points
                    )
                });
            }


            join_all(futures).await.join("\n")
        }
        None => {
            let leaderboard = get_global_leaderboard_by_points(&database);

            let leaderboard_slice = if start_index > 0
                && (start_index as usize) < leaderboard.len() - 1
                && start_index as usize + 10 < leaderboard.len() - 1
            {
                &leaderboard[start_index as usize .. start_index as usize + 10]
            } else if start_index > 0 && (start_index as usize) < leaderboard.len() - 1 {
                &leaderboard[start_index as usize ..]
            } else {
                &[]
            };

            let mut futures = Vec::new();

            for (placement, user) in leaderboard_slice.iter().enumerate() {
                futures.push(async move {
                    format!(
                        "{}: {} ({} points)",
                        placement as i32 + start_index + 1,
                        ctx.ctx.http.get_user(user.id.into()).await.unwrap().name,
                        user.points
                    )
                })
            }

            join_all(futures).await.join("\n")
        }
    };

    let header = match ctx.guild().await {
        Ok(g) => {
            format!("-- Showing {} server leaderboard --", g.name)
        }
        Err(_) => "Showing global leaderboard".to_owned(),
    };

    ctx.send_str(&format!("```r\n{}\n\n{}```", header, rankings))
        .await?;


    Ok(())
}
