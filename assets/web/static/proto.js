function updateCell(cellID, data, offset) {
    const view = new DataView(data);
    const elt = document.getElementById('cell' + cellID); //TODO modify element
    //elt.replaceChildren(); //TODO use this instead of the elt.append calls below once OBS browser source updates to Chrome 86+
    elt.innerHTML = '';
    let mainImg = document.createElement('img');
    const imgDirLen = Number(view.getBigUint64(offset));
    offset += 8;
    const imgDir = utf8decoder.decode(data.slice(offset, offset + imgDirLen));
    offset += imgDirLen;
    const imgFilenameLen = Number(view.getBigUint64(offset));
    offset += 8;
    const imgFilename = utf8decoder.decode(data.slice(offset, offset + imgFilenameLen));
    offset += imgFilenameLen;
    mainImg.setAttribute('src', '/static/img/' + imgDir + '/' + imgFilename + '.png');
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
            let countOverlay = document.createElement('span');
            countOverlay.setAttribute('class', 'count');
            countOverlay.append('' + count);
            elt.append(countOverlay);
            break;
        case 2:
            // Image
            const overlayDirLen = Number(view.getBigUint64(offset));
            offset += 8;
            const overlayDir = utf8decoder.decode(data.slice(offset, offset + overlayDirLen));
            offset += overlayDirLen;
            const overlayImgLen = Number(view.getBigUint64(offset));
            offset += 8;
            const overlayImg = utf8decoder.decode(data.slice(offset, offset + overlayImgLen));
            offset += overlayImgLen;
            let imgOverlay = document.createElement('img');
            imgOverlay.setAttribute('src', '/static/img/' + overlayDir + '/' + overlayImg + '.png');
            elt.append(imgOverlay);
            break;
        case 3:
            // Location
            const locDimmed = view.getUint8(offset++) != 0;
            const locImgLen = Number(view.getBigUint64(offset));
            offset += 8;
            const locImg = utf8decoder.decode(data.slice(offset, offset + locImgLen));
            offset += locImgLen;
            let locOverlay = document.createElement('img');
            locOverlay.setAttribute('class', locDimmed ? 'loc dimmed' : 'loc');
            locOverlay.setAttribute('src', '/static/img/xopar-images/' + locImg + '.png');
            elt.append(locOverlay);
            break;
        default:
            throw 'unexpected CellOverlay variant';
    }
    return offset;
}

const sock = new WebSocket("wss://oottracker.fenhl.net/websocket");
const utf8decoder = new TextDecoder();
const utf8encoder = new TextEncoder();

sock.binaryType = "arraybuffer";

sock.addEventListener('open', function(event) {
    const restreamMatch = window.location.pathname.match(/^\/restream\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/?$/);
    const restreamDoubleMatch = window.location.pathname.match(/^\/restream\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/with\/([0-9A-Za-z-]+)\/?$/);
    if (restreamMatch) {
        const restreamSubscription = new ArrayBuffer(1);
        new DataView(restreamSubscription).setUint8(0, 1); // ClientMessage variant: SubscribeRestream
        const restream = utf8encoder.encode(restreamMatch[1]);
        const restreamLen = new ArrayBuffer(8);
        new DataView(restreamLen).setBigUint64(0, BigInt(restream.length));
        const runner = utf8encoder.encode(restreamMatch[2]);
        const runnerLen = new ArrayBuffer(8);
        new DataView(runnerLen).setBigUint64(0, BigInt(runner.length));
        let layout;
        switch (restreamMatch[3]) {
            case 'default':
                layout = 0;
                break;
            case 'mw-expanded':
                layout = 1;
                break;
            case 'mw-collapsed':
                layout = 2;
                break;
            case 'mw-edit':
                layout = 3;
                break;
            default:
                throw 'unknown layout';
        }
        const layoutBuf = new ArrayBuffer(1);
        new DataView(layoutBuf).setUint8(0, layout);
        sock.send(new Blob([restreamSubscription, restreamLen, restream, runnerLen, runner, layoutBuf]));
    } else if (restreamDoubleMatch) {
        const doubleSubscription = new ArrayBuffer(1);
        new DataView(doubleSubscription).setUint8(0, 2); // ClientMessage variant: SubscribeDoubleRestream
        const doubleRestream = utf8encoder.encode(restreamDoubleMatch[1]);
        const doubleRestreamLen = new ArrayBuffer(8);
        new DataView(doubleRestreamLen).setBigUint64(0, BigInt(doubleRestream.length));
        const runner1 = utf8encoder.encode(restreamDoubleMatch[2]);
        const runner1len = new ArrayBuffer(8);
        new DataView(runner1len).setBigUint64(0, BigInt(runner1.length));
        const runner2 = utf8encoder.encode(restreamDoubleMatch[4]);
        const runner2len = new ArrayBuffer(8);
        new DataView(runner2len).setBigUint64(0, BigInt(runner2.length));
        let doubleLayout;
        switch (restreamDoubleMatch[3]) {
            case 'dungeon-rewards':
                doubleLayout = 0;
                break;
            default:
                throw 'unknown layout';
        }
        const doubleLayoutBuf = new ArrayBuffer(1);
        new DataView(doubleLayoutBuf).setUint8(0, doubleLayout);
        sock.send(new Blob([doubleSubscription, doubleRestreamLen, doubleRestream, runner1len, runner1, runner2len, runner2, doubleLayoutBuf]));
    }
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
            const debugLen = Number(view.getBigUint64(offset));
            offset += 8;
            const debug = utf8decoder.decode(data.slice(offset, offset + debugLen));
            offset += debugLen;
            const displayLen = Number(view.getBigUint64(offset));
            offset += 8;
            const display = utf8decoder.decode(data.slice(offset, offset + displayLen));
            offset += displayLen;
            throw display;
        case 2:
            // Init
            const numCells = Number(view.getBigUint64(offset));
            offset += 8;
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
