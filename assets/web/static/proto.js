const sock = new WebSocket("wss://oottracker.fenhl.net/websocket");
const utf8decoder = new TextDecoder();
const utf8encoder = new TextEncoder();

sock.binaryType = "arraybuffer";

function readImgDir(discrim, overlay) {
    switch (discrim) {
        case 0:
            // ImageDir::Xopar
            return overlay ? 'xopar-overlays' : 'xopar-images';
        case 1:
            // ImageDir::Extra
            return overlay ? 'extra-overlays' : 'extra-images';
        default:
            throw 'unexpected ImageDir variant';
    }
}

function makeLayoutBuf(layoutString) {
    let buf = new ArrayBuffer(1);
    switch (layoutString) {
        case 'default':
            buf = new ArrayBuffer(4);
            new DataView(buf).setUint8(0, 0); // TrackerLayout variant: Default
            new DataView(buf).setUint8(1, 0); // TrackerLayout::Default field: auto: false
            new DataView(buf).setUint8(2, 0); // TrackerLayout::Default field: meds: LightShadowSpirit
            new DataView(buf).setUint8(3, 3); // TrackerLayout::Default field: warp_songs: SpiritShadowLight
            return buf;
        case 'mw-expanded':
            new DataView(buf).setUint8(0, 1);
            return buf;
        case 'mw-collapsed':
            new DataView(buf).setUint8(0, 2);
            return buf;
        case 'mw-edit':
            new DataView(buf).setUint8(0, 3);
            return buf;
        case 'rsl-left':
            new DataView(buf).setUint8(0, 4);
            return buf;
        case 'rsl-right':
            new DataView(buf).setUint8(0, 5);
            return buf;
        case 'rsl-edit':
            new DataView(buf).setUint8(0, 6);
            return buf;
        case 'rsl-3player':
            new DataView(buf).setUint8(0, 7);
            return buf;
        case 'tsg-main-locs':
            new DataView(buf).setUint8(0, 8);
            return buf;
        case 'tsg-main-locs-edit':
            new DataView(buf).setUint8(0, 9);
            return buf;
        default:
            throw 'unknown layout';
    }
}

function sendClick(cellID, right) {
    const mwRoomMatch = window.location.pathname.match(/^\/mw\/([0-9A-Za-z-]+)\/([0-9]+)\/([0-9A-Za-z-]+)\/?$/);
    const roomMatch = window.location.pathname.match(/^\/room\/([0-9A-Za-z-]+)\/?$/);
    const restreamMatch = window.location.pathname.match(/^\/restream\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/?$/);
    let buf;
    let bufView;
    if (mwRoomMatch) {
        const clickMw = new ArrayBuffer(1);
        new DataView(clickMw).setUint8(0, 12); // ClientMessage variant: ClickMw
        const mwRoom = utf8encoder.encode(mwRoomMatch[1]);
        const mwRoomLen = new ArrayBuffer(8);
        new DataView(mwRoomLen).setBigUint64(0, BigInt(mwRoom.length));
        const world = new ArrayBuffer(1);
        new DataView(world).setUint8(0, parseInt(mwRoomMatch[2]));
        const mwLayoutBuf = makeLayoutBuf(mwRoomMatch[3]);
        buf = new ArrayBuffer(2);
        bufView = new DataView(buf);
        bufView.setUint8(0, cellID);
        bufView.setUint8(1, right ? 1 : 0);
        sock.send(new Blob([clickMw, mwRoomLen, mwRoom, world, mwLayoutBuf, buf]));
    } else if (roomMatch) {
        const clickRoom = new ArrayBuffer(1);
        new DataView(clickRoom).setUint8(0, 5); // ClientMessage variant: ClickRoom
        const room = utf8encoder.encode(roomMatch[1]);
        const roomLen = new ArrayBuffer(8);
        new DataView(roomLen).setBigUint64(0, BigInt(room.length));
        buf = new ArrayBuffer(6);
        bufView = new DataView(buf);
        bufView.setUint8(0, 0); // TrackerLayout variant: Default
        bufView.setUint8(1, 0); // TrackerLayout::Default field: auto: false
        bufView.setUint8(2, 0); // TrackerLayout::Default field: meds: LightShadowSpirit
        bufView.setUint8(3, 3); // TrackerLayout::Default field: warp_songs: SpiritShadowLight
        bufView.setUint8(4, cellID);
        bufView.setUint8(5, right ? 1 : 0);
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
        const layoutBuf = makeLayoutBuf(restreamMatch[3]);
        buf = new ArrayBuffer(2);
        bufView = new DataView(buf);
        bufView.setUint8(0, cellID);
        bufView.setUint8(1, right ? 1 : 0);
        sock.send(new Blob([clickRestream, restreamLen, restream, runnerLen, runner, layoutBuf, buf]));
    } else {
        throw 'unknown tracker type';
    }
}

function updateCell(cellID, data, offset) {
    const view = new DataView(data);
    const elt = document.getElementById('cell' + cellID);
    //elt.replaceChildren(); //TODO use this instead of the elt.append calls below once OBS browser source updates to Chrome 86+
    elt.innerHTML = '';
    let mainImg = document.createElement('img');
    const imgDir = readImgDir(view.getUint8(offset++));
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
            readImgDir(view.getUint8(offset++));
            const countImgFilenameLen = Number(view.getBigUint64(offset));
            offset += 8;
            utf8decoder.decode(data.slice(offset, offset + countImgFilenameLen));
            offset += countImgFilenameLen;
            let countOverlay = document.createElement('span');
            countOverlay.setAttribute('class', 'count');
            countOverlay.append('' + count);
            elt.append(countOverlay);
            break;
        case 2:
            // Image
            const overlayDir = readImgDir(view.getUint8(offset++), true);
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
            const locDir = readImgDir(view.getUint8(offset++));
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
    const mwRoomMatch = window.location.pathname.match(/^\/mw\/([0-9A-Za-z-]+)\/([0-9]+)\/([0-9A-Za-z-]+)\/?$/);
    const roomMatch = window.location.pathname.match(/^\/room\/([0-9A-Za-z-]+)\/?$/);
    const restreamMatch = window.location.pathname.match(/^\/restream\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/?$/);
    const restreamDoubleMatch = window.location.pathname.match(/^\/restream\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/([0-9A-Za-z-]+)\/with\/([0-9A-Za-z-]+)\/?$/);
    if (mwRoomMatch) {
        const mwSubscription = new ArrayBuffer(1);
        new DataView(mwSubscription).setUint8(0, 13); // ClientMessage variant: SubscribeMw
        const mwRoom = utf8encoder.encode(mwRoomMatch[1]);
        const mwRoomLen = new ArrayBuffer(8);
        new DataView(mwRoomLen).setBigUint64(0, BigInt(mwRoom.length));
        const world = new ArrayBuffer(1);
        new DataView(world).setUint8(0, parseInt(mwRoomMatch[2]));
        const mwLayoutBuf = makeLayoutBuf(mwRoomMatch[3]);
        sock.send(new Blob([mwSubscription, mwRoomLen, mwRoom, world, mwLayoutBuf]));
    } else if (roomMatch) {
        const roomSubscription = new ArrayBuffer(1);
        new DataView(roomSubscription).setUint8(0, 4); // ClientMessage variant: SubscribeRoom
        const room = utf8encoder.encode(roomMatch[1]);
        const roomLen = new ArrayBuffer(8);
        new DataView(roomLen).setBigUint64(0, BigInt(room.length));
        const roomLayoutBuf = new ArrayBuffer(4);
        new DataView(roomLayoutBuf).setUint8(0, 0); // TrackerLayout variant: Default
        new DataView(roomLayoutBuf).setUint8(1, 0); // TrackerLayout::Default field: auto: false
        new DataView(roomLayoutBuf).setUint8(2, 0); // TrackerLayout::Default field: meds: LightShadowSpirit
        new DataView(roomLayoutBuf).setUint8(3, 3); // TrackerLayout::Default field: warp_songs: SpiritShadowLight
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
        const restreamLayoutBuf = makeLayoutBuf(restreamMatch[3]);
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
