use std::time::Duration;

use rand::{prelude::SliceRandom, thread_rng};
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

    //Setting to true if arguments are present
    //all adds all members of the discord server to the groups
    //random shuffles the groups
    let mut all = false;
    let mut random = false;

    while !args.is_empty() {
        if let Ok(arg) = args.single::<String>(){
            if arg == "-all" {
                all = true;
            } else if arg == "-random" {
                random = true;
            } else {
                msg.channel_id.say(&ctx.http,format!("{} is not a valid argument.", arg)).await?;
            }
        }
    }

    //Stores the people to get shuffled or not
    let mut people: Vec<String> = Vec::new();

    if all {
        let guild = msg.guild_id.unwrap();
        let members = guild.members(&ctx.http, None, None).await?;
        msg.channel_id.say(&ctx.http,"-\nAdding all channel members to groups\n-").await?;
        for member in members.iter() {
            people.push(member.user.to_string());
        }
    }

    //Asking the user to input names
    msg.channel_id.say(&ctx.http, format!("{} is making {} groups.\n\
        Please enter the names to put in the groups or !stop to stop.\n\
        You may enter names one at a time or as a comma separated list.", msg.author, num_groups)).await?;
    //Taking input with up to a 10 minute delay
    let mut answer = msg.author.await_reply(&ctx).timeout(Duration::from_secs(600)).await;

    // Stops the loop and outputting the groups if the user does !stop
    // or adds more group members from user inputs
    while let Some(message) = answer {
        if message.content.as_str() == "!stop" {
            answer = None;
        } else {
            msg.channel_id.say(&ctx.http,"Adding them.").await?;

            message.content.as_str().split(",").for_each(|s| {
                people.push(String::from(s.trim()));
            });

            answer = msg.author.await_reply(&ctx).timeout(Duration::from_secs(600)).await;
        }
    }

    msg.channel_id.say(&ctx.http,"Making groups (may happen automatically after 10 minutes)...").await?;
    if random {
        people.shuffle(&mut thread_rng());
    }
    publish_groups(msg, ctx, &people, num_groups).await?;

    Ok(())
}

async fn publish_groups(msg: &Message, ctx: &Context, people: &Vec<String>, num_groups: u8) -> CommandResult {

    //Will store the groups with their index + 1 as the group name
    let mut groups: Vec<Vec<String>> = vec![vec![]; num_groups as usize];

    //Adding the people to groups
    let mut index: usize = 0;
    for person in people.iter() {
        groups[index].push(person.to_owned());
        index += 1;
        if index == num_groups as usize {
            index = 0;
        }
    }

    let mut output = String::from("");
    //Adding the grouped people to the output
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

    Ok(())
}