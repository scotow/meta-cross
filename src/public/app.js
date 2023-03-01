const CROSS = 0;
const CIRCLE = 1;

class Game {
    constructor(sign) {
        this.sign = sign;
        this.zoomed = false;
        this.playing = CROSS;
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
            gridEl.addEventListener('click', (event) => {
                if (!this.zoomed) {
                    this.zoomToSubGrid(selfX, selfY);
                    event.stopPropagation();
                }
            }, true);
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

        this.zoomOutEl = document.createElement('div');
        this.zoomOutEl.classList.add('action', 'hidden');
        this.zoomOutEl.innerText = 'Zoom out';
        this.zoomOutEl.addEventListener('click', () => {
            this.zoomed = false;
            this.zoomOutEl.classList.add('hidden');

            this.gridEl.style.transform = 'translate(-50%, -50%)';
            this.gridEl.style.transformOrigin = '50% 50%';
        });

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
        contentEl.append(this.youEl, this.zoomOutEl, this.opponentEl);
        this.footerEl.append(contentEl);
        document.body.append(this.footerEl);

        this.updatePlyingIndicator();
    }

    zoomToSubGrid(x, y) {
        this.zoomed = true;
        this.zoomOutEl.classList.remove('hidden');

        this.gridEl.style.transform = 'translate(-50%, -50%) scale(3)';
        this.gridEl.style.transformOrigin = `${x * 50}% ${y * 50}%`;
    }

    placeSignAndMove(data) {
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
            
            if (this.zoomed) {
                this.zoomToSubGrid(sub.x, sub.y);
            }
        }, 1200);
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
        signEl.classList.add(data.readUnsignedByte() ? 'circle' : 'cross');
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
                this.zoomed = false;
                this.gridEl.style.transform = 'translate(-50%, -50%)';
                this.gridEl.style.transformOrigin = '50% 50%';
                this.zoomOutEl.classList.add('hidden');
                this.updatePlyingIndicator(false);

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

const socket = new WebSocket(`${baseWebsocketUrl()}/ws`);
socket.binaryType = 'arraybuffer';
socket.addEventListener('open', () => {
    let game = null;
    socket.addEventListener('message', (event) => {
        const data = new ByteBuffer(event.data);
        switch (data.readUnsignedByte()) {
            case 0:
                game = new Game(data.readUnsignedByte());
                break;
            case 1:
                game.placeSignAndMove(data);
                break;
            case 2:
                game.placeSignAndWin(data);
                break;
        }
    });

    const data = new ByteBuffer(0, ByteBuffer.BIG_ENDIAN, true);
    data.writeUnsignedByte(0);
    socket.send(data.buffer);
});
socket.addEventListener('close', () => {
    alert('Connection lost.');
    window.location.reload();
});