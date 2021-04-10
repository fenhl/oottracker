function updateCell(cellID, data, offset) {
    const view = new DataView(data);
    const elt = document.getElementById('cell' + cellID); //TODO modify element
    //elt.replaceChildren(); //TODO use this instead of the elt.append calls below once OBS browser source updates to Chrome 86+
    elt.innerHTML = '';
    let mainImg = document.createElement('img');
    const imgFilenameLen = view.getBigUint64(offset);
    offset += 4;
    const imgFilename = utf8decoder.decode(data.slice(offset, offset + imgFilenameLen));
    offset += imgFilenameLen;
    mainImg.setAttribute('src', '/static/img/xopar-images/' + imgFilename);
    switch (view.getUint8(offset++)) {
        case 0:
            // Normal
            break;
        case 1:
            // Dimmed
            mainImg.setAttribute('class', 'dimmed');
            break;
        case 2:
            // LeftDimmed
            mainImg.setAttribute('class', 'left-dimmed');
            break;
        case 3:
            // RightDimmed
            mainImg.setAttribute('class', 'right-dimmed');
            break;
        default:
            throw 'unexpected CellStyle variant';
    }
    elt.append(mainImg);
    switch (view.getUint8(offset++)) {
        case 0:
            // None
            break;
        case 1:
            // Count
            const count = view.getUint8(offset++);
            let overlay = document.createElement('span');
            overlay.setAttribute('class', 'count');
            overlay.append('' + count);
            elt.append(overlay);
            break;
        case 2:
            // Image
            const overlayImgLen = view.getBigUint64(offset);
            offset += 4;
            const overlayImg = utf8decoder.decode(data.slice(offset, offset + overlayImgLen));
            offset += overlayImgLen;
            let overlay = document.createElement('img');
            overlay.setAttribute('src', overlayImg);
            elt.append(overlay);
            break;
        default:
            throw 'unexpected CellOverlay variant';
    }
    return offset;
}

const sock = WebSocket("wss://oottracker.fenhl.net/websocket");
const utf8decoder = new TextDecoder();
const utf8encoder = new TextEncoder();

sock.binaryType = "arraybuffer";

sock.addEventListener('open', function(event) {
    const match = window.location.pathname.match(/^\/restream\/([0-9A-Z_a-z]+)\/([0-9A-Z_a-z]+)\/([0-9A-Z_a-z]+)\/?$/);
    const subscription = new ArrayBuffer(1);
    new DataView(subscription).setUint8(0, 1); // ClientMessage variant: SubscribeRestream
    const restream = utf8encoder.encode(match[1]);
    const restreamLen = new ArrayBuffer(4);
    new DataView(restreamLen).setBigUint64(0, restream.length);
    const runner = utf8encoder.encode(match[2]);
    const runnerLen = new ArrayBuffer(4);
    new DataView(runnerLen).setBigUint64(0, runner.length);
    let layout;
    switch (match[3]) {
        case 'default':
            layout = 0;
            break;
        case 'mw-expanded':
            layout = 1;
            break;
        case 'mw-collapsed':
            layout = 2;
            break;
        default:
            throw 'unknown layout';
    }
    const layoutBuf = new ArrayBuffer(1);
    new DataView(layoutBuf).setUint8(0, layout);
    sock.send(new Blob([subscription, restreamLen, restream, runnerLen, runner, layoutBuf]));
});

sock.addEventListener('message', function(event) {
    const data = event.data;
    const view = new DataView(data);
    let offset = 0;
    switch (view.getUint8(offset++)) {
        case 0:
            // Ping
            const pong = new ArrayBuffer(1);
            new DataView(pong).setUint8(0, 0); // ClientMessage variant: Pong
            sock.send(pong);
            break;
        case 1:
            // Error
            const debugLen = view.getBigUint64(offset);
            offset += 4;
            const debug = utf8decoder.decode(data.slice(offset, offset + debugLen));
            offset += debugLen;
            const displayLen = view.getBigUint64(offset);
            offset += 4;
            const display = utf8decoder.decode(data.slice(offset, offset + displayLen));
            offset += displayLen;
            throw display;
        case 2:
            // Init
            const numCells = view.getBigUint64(offset);
            offset += 4;
            for (let cellID = 0; cellID < numCells; cellID++) {
                offset = updateCell(cellID, data, offset);
            }
        case 3:
            // Update
            const cellID = view.getUint8(offset++);
            updateCell(cellID, data, offset);
        default:
            throw 'unexpected ServerMessage variant';
    }
});
