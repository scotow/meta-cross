use std::{
    collections::HashSet,
    io::Cursor,
    ops::{Index, IndexMut},
};

use byteorder::ReadBytesExt;
use tokio::select;

use crate::player::{Command, Player};

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

    pub fn as_byte(&self) -> [u8; 2] {
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

    pub fn as_byte(&self) -> u8 {
        *self as u8
    }
}

pub struct Game {
    grid: Grid<Grid<Cell>>,
    playing: Sign,
    current_sub_grid: Option<Coord>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            grid: Grid::default(),
            playing: Sign::Cross,
            current_sub_grid: None,
        }
    }

    pub async fn run(mut self, mut players: [Player; 2]) -> [Option<Player>; 2] {
        dbg!("Game running...");
        players[0].send_command(Command::Start(Sign::Cross)).await;
        players[1].send_command(Command::Start(Sign::Circle)).await;
        loop {
            let (sign, message) = self.next_message(&mut players).await;
            match message {
                GameMessage::PlayerLeft => {
                    let [p1, p2] = players;
                    return match sign {
                        Sign::Cross => [None, Some(p2)],
                        Sign::Circle => [Some(p1), None],
                    };
                }
                GameMessage::Place(meta, sub) => {
                    if self.playing != sign {
                        continue;
                    }
                    if let Some(current) = self.current_sub_grid {
                        if meta != current {
                            continue;
                        }
                    }
                    if self.grid[meta][sub] != Cell::Unset {
                        continue;
                    }
                    self.grid[meta][sub] = Cell::Set(sign);
                    if let Some(winning_cells) = self.grid[meta].is_win(sign) {
                        self.send_command(
                            &mut players,
                            Command::PlaceAndWin(meta, sub, sign, winning_cells),
                        )
                        .await;
                        return players.map(Some);
                    } else {
                        self.send_command(&mut players, Command::PlaceAndMove(meta, sub, sign))
                            .await;
                        self.current_sub_grid = Some(sub);
                        self.playing = self.playing.opponent();
                    }
                }
            }
        }
    }

    async fn send_command(&mut self, players: &mut [Player; 2], command: Command) {
        players[0].send_command(command.clone()).await;
        players[1].send_command(command).await;
    }

    async fn next_message(&mut self, players: &mut [Player; 2]) -> (Sign, GameMessage) {
        let [p1, p2] = players;
        select! {
            command = p1.next_command() => {
                (Sign::Cross, match command {
                    Command::Place(meta, sub) => GameMessage::Place(meta, sub),
                    _ => GameMessage::PlayerLeft,
                })
            },
            command = p2.next_command() => {
                (Sign::Circle, match command {
                    Command::Place(meta, sub) => GameMessage::Place(meta, sub),
                    _ => GameMessage::PlayerLeft,
                })
            },
        }
    }
}

enum GameMessage {
    PlayerLeft,
    Place(Coord, Coord),
}
