use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::interactions::{Interaction, InteractionType},
};

use crate::{database::get_or_create_user, DatabaseConn};

pub struct InteractionHandler;

#[async_trait]
impl EventHandler for InteractionHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction.kind {
            InteractionType::ApplicationCommand => {
                let interaction_data = interaction.data.clone().unwrap();

                if interaction_data.name == "stats" {
                    let data = ctx.data.read().await;
                    let database = data.get::<DatabaseConn>().unwrap().lock().await;

                    let user = get_or_create_user(&database, interaction.member.user.id);
                    let self_user = ctx.http.get_current_user().await.unwrap();
                    let guild = ctx.http.get_guild(interaction.guild_id.0).await.unwrap();
                    let member = guild.member(&ctx, self_user.id).await.unwrap();
                    let color = member.colour(&ctx).await.unwrap();

                    interaction
                        .channel_id
                        .send_message(&ctx, |m| {
                            m.embed(|e| {
                                e.title(format!(
                                    "{}'s Stats",
                                    interaction
                                        .member
                                        .clone()
                                        .nick
                                        .unwrap_or(interaction.member.user.name.clone())
                                ))
                                .field("Points", format!("{} points", user.points), true)
                                .field(
                                    "Longest Chains",
                                    user.longest_chains.iter().enumerate().fold(
                                        String::new(),
                                        |mut str, v| {
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
                                        },
                                    ),
                                    true,
                                )
                                .color(color)
                            })
                        })
                        .await
                        .unwrap();
                }
            }
            _ => {}
        }
    }
}
