# Grouping-Bot

Grouping-Bot is a Discord bot written in Rust using [serenity](https://docs.rs/serenity/0.9.1/serenity/) designed to facilitate making groups and teams out of the members in a Discord server or other people.

## Running the Bot
To run the bot make sure to set up a bot in the Discord Developer Portal and set the environment variable "DISCORD_TOKEN" to your token. Add the bot to your server and build the source with cargo and you should be good to go.

## Usage

Commands for this bot follow the structure `!<command> [arguments]`.

| Command | Description
|---------|-------------|
| `!group [1-255] [arguments]` | Makes 1-255 groups from the names provided in following responses. |
| `!team [TeamName1] [TeamName2]... [TeamNameN] [arguments]` | Makes any number of teams based on the teams names passed as arguments. Members will then be added in following inputs. Arguments can be placed in between or before team names. |
| `!help [command]` | Displays usage instructions. |

| Argument | Command(s) | Description
|---------|-------------|------------|
| `-random` | `!group`, `!team` | Randomizes group/team assignments. |
| `-role` | `!group`, `!team` | Makes a Discord role for the group/team. |
| `-channel` | `!group`, `!team` | Makes a Discord channel for the group/team. If the role argument is also given, channels will be locked to the group/team's role. |
| `-all` | `!group`, `!team` | Adds all server members to the groups/teams being made. |
| `-size` | `!group` | Changes the number passed to the !group command to mean the number of people per team rather than the number of teams. Will put extra people on teams rather than having teams with too few people. |