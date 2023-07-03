use std::sync::Arc;

use anyhow::{Context, Result};
use envconfig::Envconfig;
use teloxide::prelude::*;
use tracing::*;

type Bot = teloxide::adaptors::Throttle<teloxide::Bot>;

#[derive(Debug, teloxide::macros::BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Commands:")]
enum Command {
    #[command(description = "send link to source code")]
    Source,
    #[command(description = "send start message")]
    Start,
    #[command(description = "send help message")]
    Help,
}

#[derive(Envconfig)]
struct Config {
    #[envconfig(
        from = "HELP_MSG",
        default = "This is a bot that allows all channel subscribers to send messages to it. Just send me a message and I'll forward it to the channel."
    )]
    pub help_msg: String,
    #[envconfig(
        from = "SOURCE_MSG",
        default = "Source code: https://github.com/averyanalex/infinity-tg-admins-bot"
    )]
    pub source_msg: String,
    #[envconfig(from = "SUBSCRIBE_MSG", default = "Please subscribe to the channel.")]
    pub subscribe_msg: String,
    #[envconfig(
        from = "SENT_MSG",
        default = "The message was successfully sent to the channel."
    )]
    pub sent_msg: String,
    #[envconfig(from = "CHANNEL_ID")]
    pub channel_id: i64,
    #[envconfig(from = "CHECK_SUBSCRIPTION", default = "true")]
    pub check_subscription: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::init_from_env()?;

    let bot = teloxide::Bot::from_env().throttle(teloxide::adaptors::throttle::Limits::default());
    let handler = dptree::entry().branch(
        Update::filter_message()
            .branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(handle_command),
            )
            .branch(dptree::endpoint(handle_message)),
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![Arc::new(config)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

#[tracing::instrument(skip(bot, config))]
async fn handle_command(bot: Bot, msg: Message, cmd: Command, config: Arc<Config>) -> Result<()> {
    let text = match cmd {
        Command::Source => &config.source_msg,
        Command::Start => &config.help_msg,
        Command::Help => &config.help_msg,
    };
    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}

#[tracing::instrument(skip(bot, config))]
async fn handle_message(bot: Bot, msg: Message, config: Arc<Config>) -> Result<()> {
    let subscribed = if config.check_subscription {
        let member = bot
            .get_chat_member(ChatId(config.channel_id), UserId(msg.chat.id.0.try_into()?))
            .await?;
        member.is_present()
    } else {
        true
    };
    if subscribed {
        bot.send_message(
            ChatId(config.channel_id),
            msg.text().context("message without text")?,
        )
        .await?;
        bot.send_message(msg.chat.id, &config.sent_msg).await?;
        info!("message sent to the channel")
    } else {
        bot.send_message(msg.chat.id, &config.subscribe_msg).await?;
        info!("user not subscribed")
    }
    Ok(())
}
