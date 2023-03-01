use std::collections::HashSet;

use axum::extract::ws::{Message, WebSocket};

use crate::game::{Coord, Sign};

#[derive(Debug)]
pub struct Player {
    websocket: WebSocket,
    did_start_last_game: bool,
}

impl Player {
    pub fn new(websocket: WebSocket) -> Self {
        Self {
            websocket,
            did_start_last_game: false,
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

    pub async fn send_command(&mut self, command: Command) {
        let payload = match command {
            Command::Start(sign) => vec![0, sign.as_byte()],
            Command::PlaceAndMove(meta, sub, sign) => [
                [1].as_slice(),
                meta.as_byte().as_slice(),
                sub.as_byte().as_slice(),
                [sign.as_byte()].as_slice(),
            ]
            .concat(),
            Command::PlaceAndWin(meta, sub, sign, winning_cells) => [
                [2].as_slice(),
                meta.as_byte().as_slice(),
                sub.as_byte().as_slice(),
                [sign.as_byte()].as_slice(),
                winning_cells
                    .into_iter()
                    .map(|coord| coord.as_byte())
                    .collect::<Vec<_>>()
                    .concat()
                    .as_slice(),
            ]
            .concat(),
            _ => unreachable!(),
        };
        self.websocket.send(Message::Binary(payload)).await.unwrap();
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    Queue,
    Start(Sign),
    Place(Coord, Coord),
    PlaceAndMove(Coord, Coord, Sign),
    PlaceAndWin(Coord, Coord, Sign, HashSet<Coord>),
    Leave,
}
