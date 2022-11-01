use std::fs::File;

use shadow_rs::shadow;
use dotenvy::dotenv;
use std::env;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::instrument;
use tracing::metadata::LevelFilter;
use twitch_irc::transport::tcp::{TCPTransport, TLS};
// use tracing::metadata::LevelFilter;
use anyhow::Result;
use tracing::{event, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::{PrivmsgMessage, ServerMessage};
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

shadow!(build);

#[tokio::main]
async fn main() {
    setup_tracing();

    // Read .env file
    dotenv().ok();

    let (mut incoming_messages, client) = get_client();

    // first thing you should do: start consuming incoming messages,
    // otherwise they will back up.
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            tokio::spawn(async move {
                message_processing_hub(message).await.unwrap();
            })
            .await
            .unwrap();
        }
    });

    // join a channel
    // This function only returns an error if the passed channel login name is malformed,
    // so in this simple case where the channel name is hardcoded we can ignore the potential
    // error with `unwrap`.
    client.join(get_chnl()).unwrap();

    // keep the tokio executor alive.
    // If you return instead of waiting the background task will exit.
    join_handle.await.unwrap();
}

/// A place messages go to be sent to their respective functions/checks
#[instrument(skip(message))]
async fn message_processing_hub(message: ServerMessage) -> Result<()> {
    match message {
        ServerMessage::ClearChat(_) => {
            event!(Level::DEBUG, "ClearChat called");
        }
        ServerMessage::ClearMsg(_) => {
            event!(Level::DEBUG, "ClearMsg called");
        }
        ServerMessage::GlobalUserState(_) => {
            event!(Level::DEBUG, "GlobalUserState called");
        }
        ServerMessage::Join(_) => {
            event!(Level::DEBUG, "Join called");
        }
        ServerMessage::Notice(_) => {
            event!(Level::DEBUG, "Notice called");
        }
        ServerMessage::Part(_) => {
            event!(Level::DEBUG, "Part called");
        }
        ServerMessage::Ping(_) => {
            event!(Level::DEBUG, "Ping called");
        }
        ServerMessage::Pong(_) => {
            event!(Level::DEBUG, "Pong called");
        }
        ServerMessage::Privmsg(pm) => pmsg_handle(pm).await?,
        ServerMessage::Reconnect(_) => {
            event!(Level::DEBUG, "Reconnect called");
        }
        ServerMessage::RoomState(_) => {
            event!(Level::DEBUG, "RoomState called");
        }
        ServerMessage::UserNotice(_) => {
            event!(Level::DEBUG, "UserNotice called");
        }
        ServerMessage::UserState(_) => {
            event!(Level::DEBUG, "UserState called");
        }
        ServerMessage::Whisper(_) => {
            event!(Level::DEBUG, "Whisper called");
        }
        _ => {
            event!(Level::DEBUG, "_ called");
        }
    }

    Ok(())
}

#[instrument(skip(message))]
async fn pmsg_handle(message: PrivmsgMessage) -> Result<()> {
    let msg_text = message.message_text;
    event!(Level::INFO, msg_text);
    // println!("{}", msg_text);

    if msg_text.starts_with(&env::var("COMMAND_PREFIX").unwrap_or_else(|_| "!".to_string())) {
        let cmd = what_command(msg_text)?;

        // If not a valid command then do nothing.
        if cmd.is_none() {
            return Ok(());
        }

        match cmd.unwrap() {
            BotCommand::Ping => {
                let (_, client) = get_client();

                client.say(get_chnl(), "Pong!".to_owned()).await.unwrap();
            }
            BotCommand::Help => todo!(),
            BotCommand::Quote => todo!(),
            BotCommand::About => {
                let (_, client) = get_client();

                let reply = format!("Version: {} - Debug mode: {}", build::PKG_VERSION, {if shadow_rs::is_debug() {String::from("On")} else {String::from("Off")}});

                client.say(get_chnl(), reply).await.unwrap();
            },
        }
    }

    Ok(())
}

fn what_command(msg: String) -> Result<Option<BotCommand>> {
    // Strip prefix from message and make lowercase
    let stripped = msg
        .strip_prefix(&env::var("COMMAND_PREFIX").unwrap_or_else(|_| "!".to_string()))
        .unwrap()
        .to_lowercase();

    // Loop over all possible commands
    for command_name in BotCommand::iter() {
        // Makre the enum name lower case
        let cmd = format!("{:?}", command_name).to_lowercase();

        // Check to see if it starts with the command and return enum
        if stripped.starts_with(&cmd) {
            println!("User ran: {cmd}");
            return Ok(Some(command_name));
        }
    }

    // Return None if no match
    Ok(None)
}

fn setup_tracing() {
    // Setup tracing
    let console_layer = tracing_subscriber::fmt::layer()
        .with_line_number(true)
        .with_ansi(true)
        .with_thread_names(true)
        .with_target(true)
        .with_filter(LevelFilter::INFO);
    let file_layer = match File::create(
        std::path::Path::new(&std::env::current_dir().unwrap()).join(&format!(
            "./{}_twitchy-mojo.log",
            chrono::offset::Local::now().timestamp()
        )),
    ) {
        Ok(handle) => {
            let file_log = tracing_subscriber::fmt::layer()
                .with_line_number(true)
                .with_ansi(false)
                .with_thread_names(true)
                .with_target(true)
                .with_writer(handle)
                .with_filter(LevelFilter::INFO);
            Some(file_log)
        }
        Err(why) => {
            eprintln!("ERROR!: Unable to create log output file: {why:?}");
            None
        }
    };

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();
}

fn get_client() -> (
    UnboundedReceiver<ServerMessage>,
    TwitchIRCClient<TCPTransport<TLS>, StaticLoginCredentials>,
) {
    let login_name = env::var("BOT_USERNAME").expect("Need a username");
    let oauth_token = env::var("OAUTH_TOKEN").expect("Need an oauth token");

    let config =
        ClientConfig::new_simple(StaticLoginCredentials::new(login_name, Some(oauth_token)));

    TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config)
}

fn get_chnl() -> String {
    env::var("TWITCH_CHANNEL").expect("Need to set TWITCH_CHANNEL in .env")
}

#[derive(Debug, EnumIter)]
enum BotCommand {
    Ping,
    Help,
    Quote,
    About,
}
