use std::time::Duration;

use linked_hash_map::LinkedHashMap;
use rand::{prelude::SliceRandom, thread_rng};
use serenity::{framework::standard::CommandError, prelude::*};
use serenity::model::prelude::*;

use serenity::framework::standard::{
    Args, CommandResult,
    macros::command,
};

use super::manager::Manager;




#[command]
async fn group(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild_id.expect("Failed to get guild_id from msg.");
    let manager = Manager::new(ctx, guild_id, msg.channel_id);

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
    let mut role = false;
    let mut channel = false;

    //Checking for flags
    while !args.is_empty() {
        if let Ok(arg) = args.single::<String>(){
            if arg == "-all" {
                all = true;
            } else if arg == "-random" {
                random = true;
            } else if arg == "-role" {
                role = true;
            } else if arg == "-channel" {
                channel = true;
            } else {
                msg.channel_id.say(&ctx.http,format!("{} is not a valid argument.", arg)).await?;
                return Err(CommandError::from("Invalid arguments."));
            }
        }
    }

    //Stores the people to get shuffled or not
    let mut people: Vec<String> = Vec::new();

    let mut teams: LinkedHashMap<String, Vec<String>> = LinkedHashMap::new();

    for i in 1..(num_groups + 1) {
        teams.insert(format!("Team #{}", i), Vec::<String>::new());
    }

    //Adding everyone to teams if all flag is active
    if all {
        let members = guild_id.members(&ctx.http, None, None).await?;
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
    //Shuffles the order of the people before team creation.
    if random {
        people.shuffle(&mut thread_rng());
    }
    
    manager.publish_teams(&mut people, &mut teams).await?;

    //Adding roles and channels if the flag was included.
    //If both role and channel flags are included, channels are exclusive to the role.
    //Needs to be after manager.publish_teams because it fills the teams up.
    if role {
        for (name, team) in teams.iter() {
            let current_role = manager.add_role(name, team).await?;
            if channel {
                let mut permissions = Permissions::all();
                permissions.remove(Permissions::ADMINISTRATOR);
                let member_permissions = PermissionOverwrite {
                    allow: permissions,
                    deny: Permissions::empty(),
                    kind: PermissionOverwriteType::Role(current_role.id),
                };

                let nonmember_permissions = PermissionOverwrite {
                    allow: Permissions::empty(),
                    deny: Permissions::all(),
                    kind: PermissionOverwriteType::Role(msg.guild(&ctx.cache).await.expect("Failed to get guild.").role_by_name("@everyone").expect("Failed to get roll \"Everyone\"").id),
                };
                manager.add_channel(name, Some(vec![&member_permissions, &nonmember_permissions])).await?;
            }
        }
    } else if channel {
        for name in teams.iter() {
            manager.add_channel(name.0, None).await?;
        }
    }

    Ok(())
}