use tokio::{
    select,
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
        oneshot,
        oneshot::Sender as OneshotSender,
    },
};

use crate::{
    game,
    player::{Command, Player},
};

pub struct Lobby {
    matchmaking: Sender<(Player, OneshotSender<Option<Player>>)>,
}

impl Lobby {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async {
            Self::matchmaking(rx).await;
        });

        Self { matchmaking: tx }
    }

    pub async fn join(&self, mut player: Player) {
        loop {
            match player.next_command().await {
                Command::Queue => {
                    if let Some(p) = self.queue(player).await {
                        player = p;
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    async fn queue(&self, player: Player) -> Option<Player> {
        let (otx, orx) = oneshot::channel();
        self.matchmaking.send((player, otx)).await.unwrap();
        orx.await.unwrap()
    }

    async fn matchmaking(mut rx: Receiver<(Player, OneshotSender<Option<Player>>)>) {
        let mut first_player: Option<(Player, OneshotSender<Option<Player>>)> = None;
        loop {
            if let Some((mut first_player, first_player_channel)) = first_player.take() {
                select! {
                    _ = first_player.next_command() => {
                        first_player_channel.send(None).unwrap();
                    },
                    second_player = rx.recv() => {
                        let (second_player, second_player_channel) = second_player.unwrap();
                        tokio::spawn(async move {
                            let mut players = [first_player, second_player];
                            let status = game::run(&mut players).await;
                            let [first_player, second_player] = players;

                            first_player_channel.send((status.is_ok() || status == Err(1)).then_some(first_player)).unwrap();
                            second_player_channel.send((status.is_ok() || status == Err(0)).then_some(second_player)).unwrap();
                        });
                    }
                }
            } else {
                first_player = Some(rx.recv().await.unwrap());
            }
        }
    }
}
