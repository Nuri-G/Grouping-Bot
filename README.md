# Grouping-Bot

Grouping-Bot is a Discord bot written in Rust designed to facilitate making groups and teams out of the members of a server or other people.

## Running the Bot
To run the bot make sure to set up a bot in the Discord Developer Portal and set the environment variable "DISCORD_TOKEN" to your token. Add the bot to your server and build the source with cargo and you should be good to go.

## Usage

Commands for this bot follow the structure `!<command> [arguments]`.

| Command | Description
|---------|-------------|
| `!group 5` | Makes 5 groups from the names provided in following responses. |
| `!team [TeamName1] [TeamName2]... [TeamNameN]` | Makes any number of teams based on the teams names passed as arguments. Members will then be added in following inputs. |
| `!help` | Displays usage instructions. |