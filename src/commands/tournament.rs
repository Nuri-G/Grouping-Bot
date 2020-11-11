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
    **You will need to advance rounds with only one participant.**
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
    // trim_tournament(&mut tournament, &mut all_games);
    
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
    let mut stop = false;
    // Stops the loop and outputting the teams if the user does `!stop`
    // or keeps updating tournament stats untill there is a winner.
    while let Some(message) = answer {
        let text = message.content.as_str();
        if text == "!stop" {
            msg.channel_id.say(&ctx.http,"Tournament has been ended.").await?;
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
                                    stop = true;
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
            if stop {
                answer = None
            } else {
                answer = msg.author.await_reply(&ctx).timeout(Duration::from_secs(6000)).await;
            }
        } else {
            answer = msg.author.await_reply(&ctx).timeout(Duration::from_secs(6000)).await;
        }
    }

    Ok(())
}

//Fills up all the tournament games with their next games.
fn fill_tournament(current_round: &mut LinkedHashMap<String, Arc<Mutex<Game>>>, round_num: u32, all_games: &mut LinkedHashMap<String, Arc<Mutex<Game>>>) {
    let round_num = round_num + 1;
    let mut next_round: LinkedHashMap<String, Arc<Mutex<Game>>> = LinkedHashMap::new();
    let len = current_round.len();
    if len > 1 {
        for (index, (_, game)) in current_round.iter_mut().enumerate() {
            let next_id = format!("{}-{}", round_num, index / 2 + 1);

            if index % 2 == 0 {
                let next_game = Arc::new(Mutex::new(Game::new(next_id.clone(), "".to_owned(), "".to_owned(), None)));
                all_games.insert(next_id.clone(), Arc::clone(&next_game));
                next_round.insert(next_id.clone(), Arc::clone(&next_game));
            }
            game.lock().unwrap().set_next(Some(Arc::clone(&next_round[&format!("{}-{}", round_num, index / 2 + 1)])));
        }
        fill_tournament(&mut next_round, round_num, all_games);
    }
}

// //Gets rid of games that will only have 1 team in them.
// fn trim_tournament(round: &mut LinkedHashMap<String, Arc<Mutex<Game>>>, all_games: &mut LinkedHashMap<String, Arc<Mutex<Game>>>) {
//     let mut next_round: LinkedHashMap<String, Arc<Mutex<Game>>> = LinkedHashMap::new();
//     if round.len() > 2 && round.len() % 2 == 1 {
//         let keys: Vec<String> = round.keys().map(|k| k.to_string()).collect();
//         let size = keys.len() as u32;
//         let log2size = 32 - (size - 1).leading_zeros() - 1;
//         let game_arc = Arc::clone(&round.get(&keys[keys.len() - 1]).unwrap());
//         let game = game_arc.lock().unwrap();
//         let next = Arc::clone(game.next_game.as_ref().unwrap());
//         let next_id = g.next_game.as_ref().unwrap().lock().unwrap().id.clone();
//         next.as_ref().lock().unwrap().id = next_id.clone();
//         // round.remove(&keys[keys.len() - 1]);
//         // round.insert(next_id.clone(), next);
//         // keys.remove(keys.len() - 1);
//         // keys.push(next_id.clone());
//         for key in keys.iter() {
//             next_round.insert(t.next_game.as_ref().unwrap().lock().unwrap().id.clone(), Arc::clone(t.next_game.as_ref().unwrap()));
//         }
//         trim_tournament(&mut next_round, all_games);
//     } else if round.len() == 2 {
//         let keys: Vec<String> = round.keys().map(|s| s.to_string()).collect();
//         let id = keys[1].clone();
//         let game_arc = Arc::clone(round.get(&id).unwrap());
//         let mut game = game_arc.as_ref().lock().unwrap();
//         let top = game.top_team.clone();
//         let bottom = game.bottom_team.clone();
//         let next_id = game.next_game.as_ref().unwrap().lock().unwrap().id.clone();
//         if top != "" && bottom == "" {
//             round.remove(&id);
//             round.insert(next_id.clone(), Arc::clone(game.next_game.as_mut().unwrap()));
//         }
//     }
// }
