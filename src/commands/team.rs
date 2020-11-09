use std::time::Duration;

use linked_hash_map::LinkedHashMap;
use rand::{prelude::SliceRandom, thread_rng};
use serenity::{client::Context, framework::standard::{Args, CommandError, CommandResult, macros::command}, model::channel::Message, model::{Permissions, channel::{PermissionOverwrite, PermissionOverwriteType}}};

use super::manager::Manager;




#[command]
#[description = "Makes named teams of people.\n\
    \n\
    You must use `!stop` to stop adding people to teams.\n\
    \n\
    The following example adds everyone in the discord server to randomly assigned teams, makes a role for each team, and makes a channel only for that role.\n
    \n\
    **Sample usage:** `!team team1 team2 team3 team4 -random -all -channel -role`"]
async fn team(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    let guild_id = msg.guild_id.expect("Failed to get guild_id from msg");
    let manager = Manager::new(ctx, guild_id, msg.channel_id);


    //Checking if the user is allowed to use the bot
    let member = msg.member(&ctx).await?;
    let member_permissions = member.permissions(&ctx.cache).await?;


    let mut teams: LinkedHashMap<String, Vec<String>> = LinkedHashMap::new();

    //Setting to true if arguments are present
    //all adds all members of the discord server to the groups
    //random shuffles the groups
    let mut all = false;
    let mut random = false;
    let mut role = false;
    let mut channel = false;

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
            } else if args.len() > 0 && &arg.as_str()[0..1] == "-" {
                msg.channel_id.say(&ctx.http,format!("{} is not a valid argument.", arg)).await?;
            } else {
                teams.insert(arg, Vec::new());
            }
        }
    }


    //Checking if the user is allowed to use the bot
    if !member_permissions.manage_roles() && role {
        msg.channel_id.say(&ctx.http,"You do not have sufficient permissions to make new roles.").await?;
        return Err(CommandError::from("Insufficient permissions for user."));
    }

    if !member_permissions.manage_channels() && channel {
        msg.channel_id.say(&ctx.http,"You do not have sufficient permissions to make new channels.").await?;
        return Err(CommandError::from("Insufficient permissions for user."));
    }


    if teams.len() == 0 {
        msg.channel_id.say(&ctx.http, "Please enter at least 1 valid team name.").await?;
        return Err(CommandError::from("Not enough teams."));
    }

    let num_teams = teams.keys().len();
    msg.channel_id.say(&ctx.http,format!("{} teams have been made.", num_teams)).await?;


    //Stores the people to get shuffled or not
    let mut people: Vec<String> = Vec::new();

    if all {
        let members = guild_id.members(&ctx.http, None, None).await?;
        msg.channel_id.say(&ctx.http,"-\nAdding all channel members to teams\n-").await?;
        for member in members.iter() {
            people.push(member.user.to_string());
        }
    }
    
    //Asking the user to input names
    msg.channel_id.say(&ctx.http, format!("{} is making {} teams.\n\
        Please enter the names to put in the teams or `!stop` to stop.\n\
        You may enter names one at a time or as a comma separated list.", msg.author, num_teams)).await?;
    //Taking input with up to a 10 minute delay
    let mut answer = msg.author.await_reply(&ctx).timeout(Duration::from_secs(600)).await;

    // Stops the loop and outputting the teams if the user does `!stop`
    // or adds more team members from user inputs
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

    msg.channel_id.say(&ctx.http,"Making teams (may happen automatically after 10 minutes)...").await?;
    //Shuffles the order of the people before team creation.
    if random {
        people.shuffle(&mut thread_rng());
    }

    manager.publish_teams(&people, &mut teams).await?;

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
