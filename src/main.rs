use std::env;

use serenity::{async_trait, framework::{StandardFramework, standard::macros::group}, model::{gateway::Ready}, prelude::*};

mod commands;

use commands::{
    group::*,
    team::*,
};
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Prints the bot's user name.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(group, team)]
struct General;

#[tokio::main]
async fn main() {
    // framework sets ! to be the command prefix and sets General as the command gorup.
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    // Sets the token to the DISCORD_TOKEN environment variable.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    // Creates a new bot client with the framework and event handler
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    // Starts the client with a single shard
    if let Err(why) = client.start_autosharded().await {
        println!("Client error: {:?}", why);
    }
}
