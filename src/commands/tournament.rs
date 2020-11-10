use std::{time::Duration, sync::{Arc, Mutex}};

use linked_hash_map::LinkedHashMap;
use rand::{prelude::SliceRandom, thread_rng};
use serenity::{client::Context, framework::standard::{Args, CommandError, CommandResult, macros::command}, model::channel::Message};

use super::game::Game;




#[command]
#[description = "Makes and runs a single elimination tournament bracket.\n\
    \n\
    The following example starts a single elimination tournament with 4 named teams.\n
    \n\
    **Sample usage:** `!tournament team1 team2 team3 team4`"]
async fn tournament(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    let mut teams: Vec<String> = vec![];

    //Setting to true if arguments are present
    //all adds all members of the discord server to the tournament
    //random shuffles the groups
    let mut random = false;
    let mut all = false;

    while !args.is_empty() {
        if let Ok(arg) = args.single::<String>(){
            if arg == "-random" {
                random = true;
            } else if arg == "-all" {
                all = true;
            } else if args.len() > 0 && &arg.as_str()[0..1] == "-" {
                msg.channel_id.say(&ctx.http,format!("{} is not a valid argument.", arg)).await?;
            } else {
                teams.push(arg);
            }
        }
    }

    if all {
        let members = msg.guild_id.unwrap().members(&ctx.http, None, None).await?;
        msg.channel_id.say(&ctx.http,"-\nAdding all channel members to the tournament\n-").await?;
        for member in members.iter() {
            teams.push(member.user.to_string());
        }
    }


    if teams.len() == 0 {
        msg.channel_id.say(&ctx.http, "Please enter at least 1 valid team name.").await?;
        return Err(CommandError::from("Not enough teams."));
    }

    let num_teams = teams.len();
    msg.channel_id.say(&ctx.http,format!("Making a tournament with {} participants.\n", num_teams)).await?;

    //Shuffles the order of the people before team creation.
    if random {
        teams.shuffle(&mut thread_rng());
    }

    let mut tournament: LinkedHashMap<String, Arc<Mutex<Game>>> = LinkedHashMap::new();
    let mut all_games: LinkedHashMap<String, Arc<Mutex<Game>>> = LinkedHashMap::new();
    let mut current_game = Arc::new(Mutex::new(Game::new("1-1".to_owned(), "".to_owned(), "".to_owned(), None)));

    for (index, team_name) in teams.iter().enumerate() {

        if index % 2 != 0 {
            let mut guard = tournament[&format!("{}-{}", 1, index / 2 + 1)].lock().expect("There was an unknown error.");
            guard.add_team(team_name.to_owned()).expect("Teams already filled.");
        } else {
            tournament.insert(format!("{}-{}", 1, index / 2 + 1), Arc::clone(&current_game));
            all_games.insert(format!("{}-{}", 1, index / 2 + 1), Arc::clone(&current_game));
            current_game = Arc::new(Mutex::new(Game::new(format!("{}-{}", 1, index / 2 + 1), "".to_owned(), "".to_owned(), None)));
            let mut guard = tournament[&format!("{}-{}", 1, index / 2 + 1)].lock().expect("There was an unknown error.");
            guard.add_team(team_name.to_owned()).expect("Teams already filled.");
        }
    }

    fill_tournament(&mut tournament, 1, &mut all_games);
    
    let mut out: String = String::new();
    for key in tournament.keys() {
        let mut nums = key.split("-");
        let round = nums.next().expect("There was an unknown error.");
        let game = nums.next().expect("There was an unknown error.");
        let top_team: String;
        let bottom_team: String;
        {
            let guard = tournament[key].lock().expect("There was an unknown error.");
            top_team = guard.top_team.clone();
            bottom_team = guard.bottom_team.clone();
        }
        out.push_str(format!("\nRound {} Game {}:\n\t{}\n\t{}\n", round, game, top_team, bottom_team).as_str());
    }
    msg.channel_id.say(&ctx.http, out).await?;


    //Asking the user to input names
    msg.channel_id.say(&ctx.http, "\nUse `!declare [round #]-[game #] [winner's name]` to set the result of a game.\n\
    If you want to end the tournament use `!stop`.").await?;
    //Taking input with up to a 100 minute delay
    let mut answer = msg.author.await_reply(&ctx).timeout(Duration::from_secs(6000)).await;

    // Stops the loop and outputting the teams if the user does `!stop`
    // or keeps updating tournament stats untill there is a winner.
    while let Some(message) = answer {
        let text = message.content.as_str();
        if text == "!stop" {
            answer = None;
        } else if text.starts_with("!declare") {
            let split: Vec<String> = text.split(" ").map(|s: &str| s.to_string()).collect();
            if split.len() == 3 {
                let id = split[1].clone();
                let already_assigned: bool;
                let has_team: bool;
                let next_id: String;
                {
                    if all_games.contains_key(&id) {
                        let mut game = all_games.get(&id).expect("Key not in all_games.").as_ref().lock().expect("There was an unknown error.");
                        if game.winner == "" {
                            match &game.next_game {
                                Some(next_game) => {
                                    let guard = next_game.as_ref().lock().expect("There was an unknown error.");
                                    next_id = guard.id.clone();
                                    if game.top_team == split[2] || game.bottom_team == split[2] {
                                        has_team = true;
                                    } else {
                                        has_team = false;
                                    }
                                }
                                None => {
                                    has_team = true;
                                    next_id = "won the tournament!".to_string();
                                }
                            }
                            already_assigned = false;
                            if has_team && !already_assigned {
                                game.winner(split[2].to_owned());
                            }
                        } else {
                            has_team = false;
                            already_assigned = true;
                            next_id = "".to_string();
                        }
                    } else {
                        already_assigned = false;
                        next_id = "".to_owned();
                        has_team = false;
                    }
                }
                if already_assigned {
                    msg.channel_id.say(&ctx.http,"A winner for this game has already been declared.").await?;
                } else if has_team {
                    if next_id == "won the tournament!" {
                        msg.channel_id.say(&ctx.http,format!("{} {}", split[2], next_id)).await?;
                    } else {
                        msg.channel_id.say(&ctx.http,format!("{}'s next game is {}", split[2], next_id)).await?;
                    }
                } else {
                    msg.channel_id.say(&ctx.http,"Please enter a valid game and team for this round.").await?;
                }

            } else {
                msg.channel_id.say(&ctx.http,"You should use the format `!declare [round #]-[game #] [winner's name]`").await?;
            }
            answer = msg.author.await_reply(&ctx).timeout(Duration::from_secs(6000)).await;
        } else {
            answer = msg.author.await_reply(&ctx).timeout(Duration::from_secs(6000)).await;
        }
    }

    Ok(())
}

//Fills up all the tournament games with their next games.
fn fill_tournament(round: &mut LinkedHashMap<String, Arc<Mutex<Game>>>, round_num: u32, all_games: &mut LinkedHashMap<String, Arc<Mutex<Game>>>) {
    let round_num = round_num + 1;
    let mut next_round: LinkedHashMap<String, Arc<Mutex<Game>>> = LinkedHashMap::new();
    let mut current_game;
    let len = round.len();
    let mut extra_person = false;
    if len > 1 {
        for (index, (_, game)) in round.iter_mut().enumerate() {
            if index % 2 == 0 && index + 1 != len {
                current_game = Arc::new(Mutex::new(Game::new(format!("{}-{}", round_num, index / 2 + 1), "".to_owned(), "".to_owned(), None)));
                all_games.insert(format!("{}-{}", round_num, index / 2 + 1), Arc::clone(&current_game));
                next_round.insert(format!("{}-{}", round_num, index / 2 + 1), Arc::clone(&current_game));
            } else if index % 2 == 0 {
                extra_person = true;
                let g = game.as_ref().lock().unwrap();
                let id = g.id.clone();
                next_round.insert(id, Arc::clone(game));
            }
            if index + 1 != len {
                let mut guard = game.lock().expect("There was an unknown error.");
                guard.set_next(Some(Arc::clone(&next_round[&format!("{}-{}", round_num, index / 2 + 1)])));
            }
        }
        if extra_person {
            let keys: Vec<String> = round.keys().map(|s| s.to_owned() ).collect();
            let game = Arc::clone(&round[&keys[round.len() - 1]]);
            let mut g = game.as_ref().lock().unwrap();
            let mut id = g.id.clone();
            round.remove(&id);
            g.id = format!("{}-{}", round_num, keys.len() / 2);
            id = g.id.clone();
            drop(g);
            round.insert(id, game);
        }
        fill_tournament(&mut next_round, round_num, all_games);
    }
}
