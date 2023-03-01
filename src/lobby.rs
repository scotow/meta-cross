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
    game::Game,
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

        Self {
            matchmaking: tx,
            // waiting: Mutex::new(None),
        }
    }

    pub async fn join(&self, mut player: Player) {
        // let (mut tx, mut rx) = ws.split();
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
            // let message = match ws.next().await {
            //     Some(Ok(message)) => message,
            //     _ => break,
            // };
            // let Message::Binary(data) = message else {
            //     break;
            // };
            // let Some((&id, _payload)) = data.split_first() else {
            //     break;
            // };

            // match id {
            //     0 => {
            //         // if let Some(waiting) = self.waiting.lock().await.take() {
            //         //     if waiting.
            //         //     tokio::spawn(async {
            //         //         self.play([waiting, ws]).await;
            //         //     });
            //         // } else {
            //         //     self.waiting.lock().await.insert(ws);
            //         // }
            //     }
            //     _ => break,
            // }
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
                            let game = Game::new();
                            let [first_player, second_player] = game.run([first_player, second_player]).await;
                            first_player_channel.send(first_player).unwrap();
                            second_player_channel.send(second_player).unwrap();
                        });
                    }
                }
            } else {
                first_player = Some(rx.recv().await.unwrap());
            }
        }
    }
}
