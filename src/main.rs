use std::{collections::HashSet, env};

use serenity::{async_trait, framework::{StandardFramework, standard::{Args, CommandGroup, CommandResult, HelpOptions, help_commands, macros::{group, help}}}, model::{channel::Message, gateway::Ready, id::UserId}, prelude::*};

mod commands;

use commands::{
    group::*,
    team::*,
    tournament::*,
};
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Prints the bot's user name.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[help]
async fn my_help(
   context: &Context,
   msg: &Message,
   args: Args,
   help_options: &'static HelpOptions,
   groups: &[&'static CommandGroup],
   owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_commands::plain(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[group]
#[commands(group, team, tournament)]
struct General;

#[tokio::main]
async fn main() {
    // framework sets ! to be the command prefix and sets General as the command gorup.
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP)
        .help(&MY_HELP);

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
