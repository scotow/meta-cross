use std::collections::HashSet;

use axum::extract::ws::{Message, WebSocket};

use crate::{
    game::{Coord, Sign},
    misc::AsBytes,
    packet,
};

#[derive(Debug)]
pub struct Player {
    websocket: WebSocket,
    pub did_start_last_game: bool,
    pub last_game_sign: Option<Sign>,
}

impl Player {
    pub fn new(websocket: WebSocket) -> Self {
        Self {
            websocket,
            did_start_last_game: true,
            last_game_sign: None,
        }
    }

    pub async fn next_command(&mut self) -> Command {
        let payload = match self.websocket.recv().await {
            Some(Ok(Message::Binary(payload))) => payload,
            _ => return Command::Leave,
        };
        let Some((&id, payload)) = payload.split_first() else {
            return Command::Leave;
        };

        match id {
            0 => Command::Queue,
            1 => {
                if payload.len() != 4 {
                    return Command::Leave;
                }
                let meta = Coord::from_raw(&payload[..2]).unwrap();
                let sub = Coord::from_raw(&payload[2..]).unwrap();
                Command::Place(meta, sub)
            }
            _ => Command::Leave,
        }
    }

    pub async fn send_command(&mut self, command: Command) -> bool {
        let payload = match command {
            Command::Start(sign, starting) => packet![0u8, sign, starting],
            Command::PlaceAndMove(meta, sub, sign) => packet![1u8, meta, sub, sign],
            Command::PlaceAndWin(meta, sub, sign, winning_cells) => {
                packet![2u8, meta, sub, sign, winning_cells]
            }
            Command::PlaceAndTie(meta, sub, sign) => packet![3u8, meta, sub, sign],
            _ => return false,
        };
        self.websocket.send(Message::Binary(payload)).await.is_ok()
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    Queue,
    Start(Sign, Sign),
    Place(Coord, Coord),
    PlaceAndMove(Coord, Coord, Sign),
    PlaceAndWin(Coord, Coord, Sign, HashSet<Coord>),
    PlaceAndTie(Coord, Coord, Sign),
    Leave,
}
