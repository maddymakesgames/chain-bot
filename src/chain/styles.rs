use std::{borrow::Cow, collections::HashMap};

use serenity::{
    client::Context,
    futures::future::join_all,
    model::{channel::Message, id::UserId},
    utils::Color,
};

use super::Chain;

pub async fn embed_style(
    chain: &Chain,
    points: &HashMap<UserId, u64>,
    message: &Message,
    ctx: &Context,
) {
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
                e.field("starter", chain.starter.display_name(), true);
                e.field("breaker", breaker, true);
                e.field(
                    "points",
                    points
                        .iter()
                        .map(|(id, p)| {
                            let member =
                                members.iter().filter(|m| &m.user.id == id).next().unwrap();
                            (member.display_name(), p)
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

pub async fn text_style(
    chain: &Chain,
    points: &HashMap<UserId, u64>,
    message: &Message,
    ctx: &Context,
) {
    let guild = message.guild(&ctx).await.unwrap();
    let breaker = message
        .member
        .clone()
        .unwrap()
        .nick
        .unwrap_or(message.author.name.clone());

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
            m.content(format!(
                "{} chain!\nStarter: {}\nBreaker: {}\nPoints:\n{}",
                chain.length,
                chain.starter.display_name(),
                &breaker,
                points.iter().fold(String::new(), |mut str, (id, i)| {
                    let member = members.iter().filter(|m| &m.user.id == id).next().unwrap();
                    str.push_str(&format!("{}: {}", member.display_name(), i));
                    str
                })
            ));
            m
        })
        .await
        .unwrap();
}

pub async fn classic_style(chain: &Chain, message: &Message, ctx: &Context) {
    message
        .channel_id
        .send_message(&ctx, |m| {
            m.content(format!(
                "That was a {} chain! <:booby:633112900382359555>",
                chain.length
            ));
            m
        })
        .await
        .unwrap();
}
