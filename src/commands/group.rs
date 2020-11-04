use std::time::Duration;

use serenity::{framework::standard::CommandError, prelude::*};
use serenity::model::prelude::*;

use serenity::framework::standard::{
    Args, CommandResult,
    macros::command,
};

#[command]
async fn group(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    //Making sure that the number of groups is between 1 and 255 inclusive
    let range_error = "Please enter between 1 and 255 groups.";
    let num_groups = args.single::<u8>().unwrap_or_else(|_| {
        0
    });

    if num_groups == 0 {
        msg.channel_id.say(&ctx.http, range_error).await?;
        return Err(CommandError::from(range_error));
    }

    //Will store the groups with their index + 1 as the group name
    let mut groups: Vec<Vec<String>> = vec![vec![]; num_groups as usize];

    //Asking the user to input groups
    msg.channel_id.say(&ctx.http, format!("{} is making {} groups.\nPlease enter the names to put in the groups or !stop to stop.", msg.author, num_groups)).await?;
    //Taking input with up to a 10 minute delay
    let mut answer = msg.author.await_reply(&ctx).timeout(Duration::from_secs(600)).await;
    //Index for the current group
    let mut group: usize = 0;

    // Stops the loop and outputting the groups if the user does !stop
    // or adds more group members from user inputs
    while let Some(message) = answer {
        if message.content.as_str() == "!stop" {
            answer = None;
            msg.channel_id.say(&ctx.http,"Groups made.").await?;
            let mut output = String::from("");

            for (number, group) in groups.iter().enumerate() {
                output.push_str(format!("Group #{}", number + 1).as_str());
                output.push('\n');
                for name in group {
                    output.push_str("    ");
                    output.push_str(name);
                    output.push('\n');
                }
                output.push('\n');
            }

            msg.channel_id.say(&ctx.http,output).await?;

        } else {
            msg.channel_id.say(&ctx.http,"Adding them.").await?;

            message.content.as_str().split(",").for_each(|s| {
                groups[group]
                    .append(&mut vec![String::from(s.trim())]);
                group += 1;
                if group > num_groups as usize - 1 {
                    group = 0
                }
            });

            answer = msg.author.await_reply(&ctx).timeout(Duration::from_secs(600)).await;
        }
    }


    Ok(())
}