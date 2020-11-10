use std::{sync::{Arc, Mutex}};




pub struct Game {
    pub id: String,
    pub top_team: String,
    pub bottom_team: String,
    pub winner: String,
    pub next_game: Option<Arc<Mutex<Game>>>,
}

impl Game {
    pub fn new(id: String, top_team: String, bottom_team: String, next_game: Option<Arc<Mutex<Game>>>) -> Self {
        Game {
            id,
            top_team,
            bottom_team,
            winner: String::new(),
            next_game,
        }
    }

    //Adds the team to itself if they are not filled. If they are filled,
    //returns an error.
    pub fn add_team(&mut self, name: String) -> Result<(), &str> {
        if self.top_team == "" {
            self.top_team = name;
        } else if self.bottom_team == "" {
            self.bottom_team = name;
        } else {
            return Err("Teams already set.");
        }
        Ok(())
    }

    pub fn set_next(&mut self, game: Option<Arc<Mutex<Game>>>) {
        self.next_game = game;
    }

    //Sets the winner for the current game and updates the participants in the next game.
    pub fn winner(&mut self, winner: String) {
        self.winner = winner;

        match &self.next_game {
            Some(game) => {
                let mut guard = game.lock().expect("There was an unknown error.");
                guard.add_team(self.winner.clone()).expect("Teams already full.");
            }
            
            None => ()
        }

    }
}
