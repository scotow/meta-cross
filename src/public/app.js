const CROSS = 0;
const CIRCLE = 1;

class Game {
    constructor(sign, startingSign) {
        this.sign = sign;
        this.zoomedMode = false;
        this.playing = startingSign;
        this.currentMetaGrid = null;
        this.createGrid();
        this.createFooter();
    }

    createGrid(container, selfX, selfY) {
        const gridEl = document.createElement('div');
        gridEl.classList.add('grid');
        gridEl.classList.add(container ? 'sub-grid' : 'meta-grid');

        for (let y = 0; y < 3; y += 1) {
            const rowEl = document.createElement('div');
            rowEl.classList.add('row');

            for (let x = 0; x < 3; x += 1) {
                const cellEl = document.createElement('div');
                cellEl.classList.add('cell');
                if (container === undefined) {
                    this.createGrid(cellEl, x, y);
                } else {
                    cellEl.addEventListener('click', () => {
                        socket.send(new Uint8Array([1, selfX, selfY, x, y]));
                    });
                }

                rowEl.append(cellEl);
                if (x < 2) {
                    const seperatorEl = document.createElement('div');
                    seperatorEl.classList.add('separator');
                    rowEl.append(seperatorEl);
                }
            }

            gridEl.append(rowEl);
            if (y < 2) {
                const seperatorEl = document.createElement('div');
                seperatorEl.classList.add('separator');
                gridEl.append(seperatorEl);
            }
        }

        if (container === undefined) {
            this.gridEl = gridEl;
            document.body.append(gridEl);
        } else {
            container.append(gridEl);
        }
    }

    createFooter() {
        this.footerEl = document.createElement('div');
        this.footerEl.classList.add('footer');

        const contentEl = document.createElement('content');
        contentEl.classList.add('content');

        this.youEl = document.createElement('div');
        this.youEl.classList.add('player');

        const youSign = document.createElement('div');
        youSign.classList.add('sign', this.sign === CROSS ? 'cross' : 'circle');

        const youLabelEl = document.createElement('div');
        youLabelEl.classList.add('label');
        youLabelEl.innerText = 'You';

        const zoomModeLabelEl = document.createElement('div');
        zoomModeLabelEl.classList.add('label');
        zoomModeLabelEl.innerText = 'Fullscreen';

        this.zoomModeEl = document.createElement('div');
        this.zoomModeEl.classList.add('action', 'hidden');
        this.zoomModeEl.addEventListener('click', () => {
            this.zoomedMode = !this.zoomedMode;
            zoomModeLabelEl.innerText = this.zoomedMode ? 'Zoomed' : 'Fullscreen';

            if (this.zoomedMode) {
                this.zoomToSubGrid();
            } else {
                this.zoomOut();
            }
        });

        const zoomModeButtonEl = document.createElement('div');
        zoomModeButtonEl.classList.add('button');
        zoomModeButtonEl.innerText = 'Toggle';

        this.opponentEl = document.createElement('div');
        this.opponentEl.classList.add('player');
        this.opponentEl.innerText = 'Opponent';

        this.opponentEl = document.createElement('div');
        this.opponentEl.classList.add('player');

        const opponentLabelEl = document.createElement('div');
        opponentLabelEl.classList.add('label');
        opponentLabelEl.innerText = 'Opponent';

        const opponenetSign = document.createElement('div');
        opponenetSign.classList.add('sign', this.sign === CROSS ? 'circle' : 'cross');

        this.youEl.append(youSign, youLabelEl);
        this.opponentEl.append(opponentLabelEl, opponenetSign);
        this.zoomModeEl.append(zoomModeLabelEl, zoomModeButtonEl);
        contentEl.append(this.youEl, this.zoomModeEl, this.opponentEl);
        this.footerEl.append(contentEl);
        document.body.append(this.footerEl);

        this.updatePlyingIndicator();
    }

    zoomToSubGrid() {
        this.gridEl.style.transform = 'translate(-50%, -50%) scale(3)';
        this.gridEl.style.transformOrigin = `${this.currentMetaGrid.x * 50}% ${this.currentMetaGrid.y * 50}%`;
    }

    zoomOut() {
        this.gridEl.style.transform = 'translate(-50%, -50%)';
        this.gridEl.style.transformOrigin = '50% 50%';
    }

    placeSignAndMove(data) {
        this.zoomModeEl.classList.remove('hidden');

        const meta = {
            x: data.readUnsignedByte(),
            y: data.readUnsignedByte(),
        };
        this.currentMetaGrid = meta;

        const sub = {
            x: data.readUnsignedByte(),
            y: data.readUnsignedByte(),
        };
        const cellEl = document.querySelectorAll('.sub-grid .cell')[(meta.y * 3 + meta.x) * 9 + sub.y * 3 + sub.x];
        const signEl = document.createElement('div');
        signEl.classList.add(data.readUnsignedByte() ? 'circle' : 'cross');
        cellEl.append(signEl);
        this.playing ^= 1;

        setTimeout(() => {
            this.gridEl.classList.add('fadded');
            const subGrids = document.querySelectorAll('.sub-grid');
            for (let i = 0; i < subGrids.length; i++) {
                subGrids[i].classList.toggle('fadded', i !== sub.y * 3 + sub.x);
            }
            this.updatePlyingIndicator();
            
            this.currentMetaGrid = sub;
            if (this.zoomedMode) {
                this.zoomToSubGrid();
            }
        }, 800);
    }

    placeSignAndWin(data) {
        const meta = {
            x: data.readUnsignedByte(),
            y: data.readUnsignedByte(),
        };
        const sub = {
            x: data.readUnsignedByte(),
            y: data.readUnsignedByte(),
        };
        const cellEl = document.querySelectorAll('.sub-grid .cell')[(meta.y * 3 + meta.x) * 9 + sub.y * 3 + sub.x];
        const signEl = document.createElement('div');
        const sign = data.readUnsignedByte();
        signEl.classList.add(sign ? 'circle' : 'cross');
        cellEl.append(signEl);

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
                this.updatePlyingIndicator(false);
                this.zoomedMode = false;
                this.zoomOut();

                const gameStatusLabelEl = document.createElement('div');
                gameStatusLabelEl.classList.add('label');
                gameStatusLabelEl.innerText = sign === this.sign ? 'You win' : 'You lose';
        
                const gameStatusEl = document.createElement('div');
                gameStatusEl.classList.add('action');
                gameStatusEl.addEventListener('click', () => {
                    this.gridEl.remove();
                    this.footerEl.remove();
                    queue();
                });
        
                const gameStatusButtonEl = document.createElement('div');
                gameStatusButtonEl.classList.add('button');
                gameStatusButtonEl.innerText = 'Play again';
        
                gameStatusEl.append(gameStatusLabelEl, gameStatusButtonEl);
                this.zoomModeEl.replaceWith(gameStatusEl);

                setTimeout(() => {
                    document.querySelectorAll('.grid').forEach((el) => el.classList.remove('fadded'));
                }, 800);
            }, 1000);
        }, 600);
    }

    updatePlyingIndicator(state) {
        if (typeof state === 'boolean') {
            this.youEl.classList.toggle('playing', state);
            this.opponentEl.classList.toggle('playing', state);
        } else {
            this.youEl.classList.toggle('playing', this.playing === this.sign);
            this.opponentEl.classList.toggle('playing', this.playing !== this.sign);
        }
    }
}

function baseWebsocketUrl() {
    return `${window.location.protocol.slice(0, -1) === 'https' ? 'wss' : 'ws'}://${window.location.host}`;
}

function queue() {
    document.getElementById('queue').hidden = false;
    const data = new ByteBuffer(0, ByteBuffer.BIG_ENDIAN, true);
    data.writeUnsignedByte(0);
    socket.send(data.buffer);
}

const socket = new WebSocket(`${baseWebsocketUrl()}/ws`);
socket.binaryType = 'arraybuffer';
socket.addEventListener('open', () => {
    let game = null;
    socket.addEventListener('message', (event) => {
        const data = new ByteBuffer(event.data);
        switch (data.readUnsignedByte()) {
            case 0:
                document.getElementById('queue').hidden = true;
                game = new Game(data.readUnsignedByte(), data.readUnsignedByte());
                break;
            case 1:
                game.placeSignAndMove(data);
                break;
            case 2:
                game.placeSignAndWin(data);
                break;
        }
    });

    queue();
});
socket.addEventListener('close', () => {
    alert('Connection lost.');
    window.location.reload();
});