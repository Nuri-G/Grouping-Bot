# Grouping-Bot

Grouping-Bot is a Discord bot written in Rust designed to facilitate making groups, teams, and tournaments out of the members in a Discord server or other people. It uses [serenity](https://docs.rs/serenity/0.9.1/serenity/) to interact with the Discord API. Commands are one file each and rely on the Manager struct and implementation in manager.rs to manage the server's roles and channels. The tournament command also uses the Game struct in game.rs to build the tournament's data structure.

## Running the Bot

To run the bot yourself, set up a bot in the [Discord Developer Portal](https://discord.com/developers/) and set the environment variable on your machine called "DISCORD_TOKEN" to the token in the portal. Add the bot to your server and run the code with cargo and you should be good to go.

Alternatively, you can use [this link](https://discord.com/api/oauth2/authorize?client_id=773009707794300929&permissions=8&scope=bot) to add the bot to your server.

## Usage

Commands for this bot follow the structure `!<command> [arguments]`.

| Command | Description
|---------|-------------|
| `!group [1-255] [arguments]` | Makes 1-255 groups from the names provided in following responses. |
| `!team [TeamName1] [TeamName2]... [TeamNameN] [arguments]` | Makes any number of teams based on the teams names passed as arguments. Members will then be added in following inputs. Arguments can be placed in between or before team names. |
| `!tournament [TeamName1] [TeamName2]... [TeamNameN] [arguments]` | Makes and runs a single elimination tournament bracket from any number of teams. |
| `!help [command]` | Displays usage instructions. |

| Argument | Command(s) | Description
|---------|-------------|------------|
| `-all` | `!group`, `!team`, `!tournament` | Adds all server members to the command. |
| `-random` | `!group`, `!team`, `!tournament` | Randomizes the order of people. |
| `-role` | `!group`, `!team` | Makes a Discord role for the group/team. |
| `-channel` | `!group`, `!team` | Makes a Discord channel for the group/team. If the role argument is also given, channels will be locked to the group/team's role. |
| `-size` | `!group` | Changes the number passed to the !group command to mean the number of people per team rather than the number of teams. Will put extra people on teams rather than having teams with too few people. |