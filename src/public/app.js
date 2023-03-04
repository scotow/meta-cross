const CROSS = 0;
const CIRCLE = 1;

class Game {
    constructor(sign, startingSign) {
        this.sign = sign;
        this.playing = startingSign;
        this.currentMetaGrid = null;
        this.finished = false;

        document.getElementById('game').classList.remove('hidden');
        document.getElementById('you').classList.remove('cross', 'circle');
        document.getElementById('you').classList.add(sign === CROSS ? 'cross' : 'circle');
        document.getElementById('opponent').classList.remove('cross', 'circle');
        document.getElementById('opponent').classList.add(sign === CROSS ? 'circle' : 'cross');
        document.querySelectorAll('.footer .action').forEach((actionEl) => {
            actionEl.classList.add('hidden');
        });
        this.updatePlyingIndicator();
        document.getElementById('footer').classList.remove('hidden');
    }

    placeSign(data) {
        const meta = {
            x: data.readUnsignedByte(),
            y: data.readUnsignedByte(),
        };
        const sub = {
            x: data.readUnsignedByte(),
            y: data.readUnsignedByte(),
        };
        this.currentMetaGrid = meta;

        const signEl = document.createElement('div');
        const sign = data.readUnsignedByte();
        signEl.classList.add(sign ? 'circle' : 'cross');
        document.querySelectorAll('.sub-grid .cell')[(meta.y * 3 + meta.x) * 9 + sub.y * 3 + sub.x].append(signEl);

        return {
            meta,
            sub,
            sign,
        };
    }

    placeSignAndMove(data) {
        const { sub } = this.placeSign(data);
        this.playing ^= 1;

        setTimeout(() => {
            document.getElementById('game').classList.add('fadded');
            document.querySelectorAll('.sub-grid').forEach((subGridEl, i) => {
                subGridEl.classList.toggle('fadded', i !== sub.y * 3 + sub.x);
            });
            this.updatePlyingIndicator();
            
            this.currentMetaGrid = sub;
        }, 800);
    }

    placeSignAndWin(data) {
        const { meta, sign } = this.placeSign(data);
        this.finished = true;

        setTimeout(() => {
            let cellEls = document.querySelectorAll('.sub-grid .cell');
            while (data.available) {
                const cell = {
                    x: data.readUnsignedByte(),
                    y: data.readUnsignedByte(),
                };
                cellEls[(meta.y * 3 + meta.x) * 9 + cell.y * 3 + cell.x].classList.add('blink');
            }
            setTimeout(() => {
                this.endGame(sign === this.sign ? 'win' : 'lose');
            }, 1000);
        }, 600);
    }

    placeSignAndTie(data) {
        this.placeSign(data);
        this.finished = true;
        
        setTimeout(() => {
            this.endGame('tie');
        }, 600);
    }

    winByForfeit() {
        this.finished = true;
        this.endGame('forfeit');
    }

    endGame(result) {
        this.updatePlyingIndicator(false);
        document.querySelectorAll('.footer .action').forEach((actionEl) => {
            actionEl.classList.add('hidden');
        });
        document.getElementById(result).classList.remove('hidden');

        setTimeout(() => {
            document.querySelectorAll('.grid').forEach((gridEl) => gridEl.classList.remove('fadded'));
        }, 800);
    }

    updatePlyingIndicator(state) {
        if (typeof state === 'boolean') {
            document.getElementById('you').classList.toggle('playing', state);
            document.getElementById('opponent').classList.toggle('playing', state);
        } else {
            document.getElementById('you').classList.toggle('playing', this.playing === this.sign);
            document.getElementById('opponent').classList.toggle('playing', this.playing !== this.sign);
        }
    }
}

function baseWebsocketUrl() {
    return `${window.location.protocol.slice(0, -1) === 'https' ? 'wss' : 'ws'}://${window.location.host}`;
}

function queue() {
    queueTimeout = setTimeout(() => {
        document.getElementById('queue').classList.remove('hidden');
    }, 500);
    socket.send(new Uint8Array([0]));
}

let queueTimeout = null;
const socket = new WebSocket(`${baseWebsocketUrl()}/ws`);
socket.binaryType = 'arraybuffer';
socket.addEventListener('open', () => {
    let game = null;

    document.querySelectorAll('.sub-grid .cell').forEach((cellEl, i) => {
        cellEl.addEventListener('click', () => {
            if (game && !game.finished) {
                socket.send(new Uint8Array([
                    1,
                    Math.floor(i / 9 % 3),
                    Math.floor(i / 27),
                    i % 3,
                    Math.floor(i % 9 / 3),
                ]));
            }
        });
    });

    document.querySelectorAll('.footer .action').forEach((actionEl) => {
        actionEl.addEventListener('click', () => {
            document.getElementById('game').classList.add('hidden');
            document.getElementById('footer').classList.add('hidden');
            setTimeout(() => {
                document.querySelectorAll('.cell > .cross, .cell > .circle').forEach((el) => {
                    el.remove();
                });
                queue();
            }, 2000);
        });
    });
    
    socket.addEventListener('message', (event) => {
        const data = new ByteBuffer(event.data);
        switch (data.readUnsignedByte()) {
            case 0:
                clearTimeout(queueTimeout);
                if (document.getElementById('queue').classList.contains('hidden')) {
                    game = new Game(data.readUnsignedByte(), data.readUnsignedByte());
                } else {
                    document.getElementById('queue').classList.add('hidden');
                    setTimeout(() => {
                        game = new Game(data.readUnsignedByte(), data.readUnsignedByte());
                    }, 500);
                }            
                break;
            case 1:
                game.placeSignAndMove(data);
                break;
            case 2:
                game.placeSignAndWin(data);
                break;
            case 3:
                game.placeSignAndTie(data);
                break;
            case 4:
                game.winByForfeit();
                break;
        }
    });

    queue();
});
socket.addEventListener('close', () => {
    alert('Connection lost.');
    window.location.reload();
});