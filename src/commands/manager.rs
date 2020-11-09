

use linked_hash_map::LinkedHashMap;
use serenity::{Error, client::Context, model::{channel::{GuildChannel, PermissionOverwrite}, guild::Role, id::{ChannelId, GuildId}}};

// A manager should make interacting with a discord server easier.
pub struct Manager<'a> {
    ctx: &'a Context,
    guild_id: GuildId,
    channel_id: ChannelId,
}

impl<'a> Manager<'a> {
    // Constructor to make a new manager
    // ctx - The serenity Context to use
    // guild_id - The serenity GuildId that should be interacted with
    // channel_id - The serenity ChannelId that should be interacted with
    pub fn new(ctx: &'a Context, guild_id: GuildId, channel_id: ChannelId) -> Self {
        Manager {
            ctx,
            guild_id,
            channel_id
        }
    }

    // Adds a new role 'name' and assigns it to all the user values in people if they are in the server.
    // Works for both regular user names and user id strings.
    pub async fn add_role(&self, name: &String, people: &Vec<String>) -> Result<Role, Error> {
        let role = self.guild_id.create_role(&self.ctx.http, |r| r
            .mentionable(true)
            .name(name)).await?;
        let mut members = self.guild_id.members(&self.ctx.http, None, None).await?;
        for member in members.iter_mut() {
            if people.contains(&member.to_string().replace("@", "@!")) || people.contains(&member.display_name().to_string()) {
                member.add_role(&self.ctx.http, role.id).await?;
            }
        }
        Ok(role)
    }

    // Adds a new channel 'name' and adds all the users in people if they are in the server.
    // Works for both regular user names and user id strings.
    pub async fn add_channel(&self, name: &String, permissions: Option<Vec<&PermissionOverwrite>>) -> Result<GuildChannel, Error> {
        let channel = self.guild_id.create_channel(&self.ctx.http, |c| c
            .name(name)).await;
        match &channel {
            Ok(guild_channel) => {
                match permissions {
                    Some(permissions) => {
                        for permission in permissions.iter() {
                            guild_channel.id.create_permission(&self.ctx.http, &permission).await?;
                        }
                    }
                    None => {}
                }
            }
            Err(_) => {}
        }
        channel
    }

    // Takes in people and teams, and assigns all the people to teams and prints the result to the server.
    // Modifies the teams LinkedHashMap so that it contains all the people in people.
    // teams must already contain the team names to be assigned to.
    pub async fn publish_teams(&self, people: &Vec<String>, teams: &mut LinkedHashMap<String, Vec<String>>) -> Result<(), Error> {


        //Adding the people to groups
        let mut index: usize = 0;
    
        let team_keys: Vec<String> = teams.keys().map(|k| k.to_owned()).collect();
        let num_teams = team_keys.len();
    
        for person in people.iter() {
            teams.get_mut(&team_keys[index]).expect("Failed to get team from key").push(person.to_owned());
            index += 1;
            if index == num_teams as usize {
                index = 0;
            }
        }
    
        let mut output = String::from("");
        //Adding the grouped people to the output
        for (team_name, team) in teams.iter() {
            output.push_str(format!("{}:", team_name).as_str());
            output.push('\n');
            for name in team.iter() {
                output.push_str("    ");
                output.push_str(name);
                output.push('\n');
            }
            output.push('\n');
        }
    
        self.channel_id.say(&self.ctx.http, output).await?;
    
        Ok(())
    }

}
