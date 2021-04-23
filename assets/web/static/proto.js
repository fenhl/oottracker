const sock = new WebSocket("wss://oottracker.fenhl.net/websocket");
const utf8decoder = new TextDecoder();
const utf8encoder = new TextEncoder();

sock.binaryType = "arraybuffer";

function restreamLayoutID(layoutString) {
    switch (layoutString) {
        case 'default':
            return 0;
        case 'mw-expanded':
            return 1;
        case 'mw-collapsed':
            return 2;
        case 'mw-edit':
            return 3;
        case 'rsl-left':
            return 4;
        case 'rsl-right':
            return 5;
        case 'rsl-edit':
            return 6;
        default:
            throw 'unknown layout';
    }
}

function sendClick(cellID, right) {
    const roomMatch = window.location.pathname.match(/^\/room\/([0-9A-Za-z-]+)\/?$/);
    const restreamMatch = window.location.pathname.match(/^\/restream\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/?$/);
    const buf = new ArrayBuffer(3);
    const bufView = new DataView(buf);
    if (roomMatch) {
        const clickRoom = new ArrayBuffer(1);
        new DataView(clickRoom).setUint8(0, 5); // ClientMessage variant: ClickRoom
        const room = utf8encoder.encode(roomMatch[1]);
        const roomLen = new ArrayBuffer(8);
        new DataView(roomLen).setBigUint64(0, BigInt(room.length));
        bufView.setUint8(0, 0); // TrackerLayout variant: Default
        bufView.setUint8(1, cellID);
        bufView.setUint8(2, right ? 1 : 0);
        sock.send(new Blob([clickRoom, roomLen, room, buf]));
    } else if (restreamMatch) {
        const clickRestream = new ArrayBuffer(1);
        new DataView(clickRestream).setUint8(0, 3); // ClientMessage variant: ClickRestream
        const restream = utf8encoder.encode(restreamMatch[1]);
        const restreamLen = new ArrayBuffer(8);
        new DataView(restreamLen).setBigUint64(0, BigInt(restream.length));
        const runner = utf8encoder.encode(restreamMatch[2]);
        const runnerLen = new ArrayBuffer(8);
        new DataView(runnerLen).setBigUint64(0, BigInt(runner.length));
        bufView.setUint8(0, restreamLayoutID(restreamMatch[3]));
        bufView.setUint8(1, cellID);
        bufView.setUint8(2, right ? 1 : 0);
        sock.send(new Blob([clickRestream, restreamLen, restream, runnerLen, runner, buf]));
    } else {
        throw 'unknown tracker type';
    }
}

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
    switch (view.getUint8(offset++)) { // style
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
    switch (view.getUint8(offset++)) { // overlay
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
            const locDirLen = Number(view.getBigUint64(offset));
            offset += 8;
            const locDir = utf8decoder.decode(data.slice(offset, offset + locDirLen));
            offset += locDirLen;
            const locImgLen = Number(view.getBigUint64(offset));
            offset += 8;
            const locImg = utf8decoder.decode(data.slice(offset, offset + locImgLen));
            offset += locImgLen;
            let locStyle;
            switch (view.getUint8(offset++)) { // style
                case 0:
                    // Normal
                    locStyle = 'loc';
                    break;
                case 1:
                    // Dimmed
                    locStyle = 'loc dimmed';
                    break;
                case 2:
                    // Mq
                    locStyle = 'loc mq';
                    break;
                default:
                    throw 'unexpected LocationStyle variant';
            }
            let locOverlay = document.createElement('img');
            locOverlay.setAttribute('class', locStyle);
            locOverlay.setAttribute('src', '/static/img/' + locDir + '/' + locImg + '.png');
            elt.append(locOverlay);
            break;
        default:
            throw 'unexpected CellOverlay variant';
    }
    if (elt.hasAttribute('href')) {
        elt.addEventListener('click', function(event) { event.preventDefault(); sendClick(cellID, false); return false; }, false);
        elt.addEventListener('contextmenu', function(event) { event.preventDefault(); sendClick(cellID, true); return false; }, false);
        elt.removeAttribute('href');
    }
    return offset;
}

sock.addEventListener('open', function(event) {
    const roomMatch = window.location.pathname.match(/^\/room\/([0-9A-Za-z-]+)\/?$/);
    const restreamMatch = window.location.pathname.match(/^\/restream\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/?$/);
    const restreamDoubleMatch = window.location.pathname.match(/^\/restream\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/with\/([0-9A-Za-z-]+)\/?$/);
    if (roomMatch) {
        const roomSubscription = new ArrayBuffer(1);
        new DataView(roomSubscription).setUint8(0, 4); // ClientMessage variant: SubscribeRoom
        const room = utf8encoder.encode(roomMatch[1]);
        const roomLen = new ArrayBuffer(8);
        new DataView(roomLen).setBigUint64(0, BigInt(room.length));
        const roomLayoutBuf = new ArrayBuffer(1);
        new DataView(roomLayoutBuf).setUint8(0, 0); // TrackerLayout variant: Default
        sock.send(new Blob([roomSubscription, roomLen, room, roomLayoutBuf]));
    } else if (restreamMatch) {
        const restreamSubscription = new ArrayBuffer(1);
        new DataView(restreamSubscription).setUint8(0, 1); // ClientMessage variant: SubscribeRestream
        const restream = utf8encoder.encode(restreamMatch[1]);
        const restreamLen = new ArrayBuffer(8);
        new DataView(restreamLen).setBigUint64(0, BigInt(restream.length));
        const runner = utf8encoder.encode(restreamMatch[2]);
        const runnerLen = new ArrayBuffer(8);
        new DataView(runnerLen).setBigUint64(0, BigInt(runner.length));
        const restreamLayoutBuf = new ArrayBuffer(1);
        new DataView(restreamLayoutBuf).setUint8(0, restreamLayoutID(restreamMatch[3]));
        sock.send(new Blob([restreamSubscription, restreamLen, restream, runnerLen, runner, restreamLayoutBuf]));
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
            break;
        case 3:
            // Update
            const cellID = view.getUint8(offset++);
            updateCell(cellID, data, offset);
            break;
        default:
            throw 'unexpected ServerMessage variant';
    }
});
