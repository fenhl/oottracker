// The constants above are generated from Rust code in crate/oottracker-utils/src/release.rs. If they're missing, you have the wrong file.

const VERSION = 4; //TODO compare with version in Rust code
var RAM_INIT_PACKET_LENGTH = 1;
for (var i = 0; i < RAM_RANGES.length; i++) {
    RAM_INIT_PACKET_LENGTH += RAM_RANGES[i][1];
}

var sock = new Socket();
sock.on('close', function() {
    alert('connection to oottracker lost');
    throw 'connection to oottracker lost';
});
sock.connect({host: "127.0.0.1", port: TCP_PORT}, function() {
    const handshake = new ArrayBuffer(1);
    new DataView(handshake).setUint8(0, VERSION);
    sock.write(new Buffer(new Uint8Array(handshake)), function() {
        console.log('Connected to OoT Tracker');
        var rawRam = null;
        events.ondraw(function() {
            var changed = true;
            if (rawRam === null) {
                rawRam = [];
                for (var i = 0; i < RAM_RANGES.length; i++) {
                    rawRam.push(mem.getblock(ADDR_ANY_RDRAM.start + RAM_RANGES[i][0], RAM_RANGES[i][1]));
                }
            } else {
                changed = false;
                for (var i = 0; i < RAM_RANGES.length; i++) {
                    const newRange = mem.getblock(ADDR_ANY_RDRAM.start + RAM_RANGES[i][0], RAM_RANGES[i][1]);
                    if (newRange != rawRam[i]) {
                        rawRam[i] = newRange;
                        changed = true;
                    }
                }
            }
            if (!changed) { return; }
            const ramData = new ArrayBuffer(RAM_INIT_PACKET_LENGTH);
            new DataView(ramData).setUint8(0, 4); // Packet variant: RamInit //TODO send deltas after the first frame
            const ramDataByteArray = new Uint8Array(ramData);
            var offset = 1;
            for (var i = 0; i < RAM_RANGES.length; i++) {
                ramDataByteArray.set(new Uint8Array(rawRam[i]), offset);
                offset += RAM_RANGES[i][1];
            }
            sock.write(new Buffer(ramDataByteArray));
        });
    });
});
