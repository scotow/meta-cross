use std::{
    collections::HashSet,
    io::Cursor,
    ops::{Index, IndexMut},
};

use byteorder::ReadBytesExt;
use rand::random;
use tokio::select;

use crate::{
    misc::AsBytes,
    player::{Command, Player},
};

const GRID_SIZE: usize = 3;

#[derive(Default, Debug)]
struct Grid<S> {
    cells: [[S; GRID_SIZE]; GRID_SIZE],
}

impl Grid<Cell> {
    fn is_win(&self, sign: Sign) -> Option<HashSet<Coord>> {
        let winning_cells = [
            ((0, 0), (0, 1), (1, 0), GRID_SIZE),
            ((0, 0), (1, 0), (0, 1), GRID_SIZE),
            ((0, 0), (1, 1), (0, 0), 1),
            ((2, 0), (-1, 1), (0, 0), 1),
        ]
        .into_iter()
        .flat_map(|((sx, sy), (dx, dy), (sdx, sdy), n)| {
            (0..n as isize).map(move |i| {
                (0..GRID_SIZE as isize).map(move |j| Coord {
                    x: (sx + sdx * i + dx * j) as usize,
                    y: (sy + sdy * i + dy * j) as usize,
                })
            })
        })
        .filter(|series| series.clone().all(|c| self[c] == Cell::Set(sign)))
        .flatten()
        .collect::<HashSet<_>>();

        (!winning_cells.is_empty()).then(|| winning_cells)
    }

    fn is_tie(&self) -> bool {
        self.cells
            .iter()
            .flatten()
            .all(|cell| matches!(cell, Cell::Set(_)))
    }
}

impl<S> Index<Coord> for Grid<S> {
    type Output = S;

    fn index(&self, index: Coord) -> &Self::Output {
        &self.cells[index.y][index.x]
    }
}

impl<S> IndexMut<Coord> for Grid<S> {
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output {
        &mut self.cells[index.y][index.x]
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct Coord {
    x: usize,
    y: usize,
}

impl Coord {
    pub fn from_raw(data: &[u8]) -> Option<Self> {
        let mut data = Cursor::new(data);
        let x = data.read_u8().ok()? as usize;
        let y = data.read_u8().ok()? as usize;
        if x < GRID_SIZE && y < GRID_SIZE {
            Some(Self { x, y })
        } else {
            None
        }
    }
}

impl AsBytes for Coord {
    type Output = [u8; 2];

    fn as_bytes(&self) -> Self::Output {
        [self.x as u8, self.y as u8]
    }
}

#[derive(Eq, PartialEq, Default, Copy, Clone, Debug)]
enum Cell {
    #[default]
    Unset,
    Set(Sign),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Sign {
    Cross,
    Circle,
}

impl Sign {
    fn opponent(self) -> Self {
        match self {
            Sign::Cross => Sign::Circle,
            Sign::Circle => Sign::Cross,
        }
    }

    fn random() -> Self {
        if random() {
            Self::Cross
        } else {
            Self::Circle
        }
    }
}

impl AsBytes for Sign {
    type Output = [u8; 1];

    fn as_bytes(&self) -> Self::Output {
        [*self as u8]
    }
}

pub async fn run(players: &mut [Player; 2]) -> Result<(), usize> {
    if let Err(leaver) = inner_run(players).await {
        players[leaver ^ 1]
            .send_command(Command::WinByForfeit)
            .await;
        return Err(leaver);
    }
    Ok(())
}

async fn inner_run(players: &mut [Player; 2]) -> Result<(), usize> {
    let mut grid = Grid::<Grid<Cell>>::default();
    let mut current_sub_grid = None;

    let starting = if players[0].did_start_last_game == players[1].did_start_last_game {
        random()
    } else {
        players[0].did_start_last_game
    } as usize;
    let first_player_sign = match (players[0].last_game_sign, players[1].last_game_sign) {
        (s1, s2) if s1 == s2 => Sign::random(),
        (None, Some(s2)) => s2.opponent(),
        (Some(s1), _) => s1,
        _ => unreachable!(),
    };
    players[starting].did_start_last_game = true;
    players[starting ^ 1].did_start_last_game = false;
    players[0].last_game_sign = Some(first_player_sign);
    players[1].last_game_sign = Some(first_player_sign.opponent());
    let mut playing = players[starting].last_game_sign.unwrap();

    players[0]
        .send_command(Command::Start(first_player_sign, playing))
        .await;
    players[1]
        .send_command(Command::Start(first_player_sign.opponent(), playing))
        .await;
    loop {
        let PlaceRequest {
            player_index,
            meta,
            sub,
        } = next_message(players).await?;
        let sign = players[player_index].last_game_sign.unwrap();
        if playing != sign {
            continue;
        }
        if let Some(current) = current_sub_grid {
            if meta != current {
                continue;
            }
        }
        if grid[meta][sub] != Cell::Unset {
            continue;
        }

        grid[meta][sub] = Cell::Set(sign);
        if let Some(winning_cells) = grid[meta].is_win(sign) {
            return send_command(
                players,
                Command::PlaceAndWin(meta, sub, sign, winning_cells),
            )
            .await;
        } else if grid[sub].is_tie() {
            return send_command(players, Command::PlaceAndTie(meta, sub, sign)).await;
        } else {
            send_command(players, Command::PlaceAndMove(meta, sub, sign)).await?;
            current_sub_grid = Some(sub);
            playing = playing.opponent();
        }
    }
}

async fn next_message(players: &mut [Player; 2]) -> Result<PlaceRequest, usize> {
    let [p1, p2] = players;
    select! {
        command = p1.next_command() => {
            match command {
                Command::Place(meta, sub) => Ok(PlaceRequest {
                    player_index: 0,
                    meta, sub,
                }),
                _ => Err(0),
            }
        },
        command = p2.next_command() => {
            match command {
                Command::Place(meta, sub) => Ok(PlaceRequest {
                    player_index: 1,
                    meta, sub,
                }),
                _ => Err(1),
            }
        },
    }
}

async fn send_command(players: &mut [Player; 2], command: Command) -> Result<(), usize> {
    for i in 0..2 {
        if !players[i].send_command(command.clone()).await {
            return Err(i);
        }
    }
    Ok(())
}

struct PlaceRequest {
    player_index: usize,
    meta: Coord,
    sub: Coord,
}
