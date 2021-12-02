use {
    std::{
        collections::HashMap,
        future::Future,
        io::Write,
        marker::Unpin,
        pin::Pin,
    },
    async_proto::{
        Protocol,
        ReadError,
        WriteError,
    },
    byteorder::{
        BigEndian,
        ByteOrder as _,
    },
    serde::{
        Deserialize,
        Serialize,
    },
    tokio::io::{
        AsyncRead,
        AsyncReadExt as _,
        AsyncWrite,
        AsyncWriteExt as _,
    },
    ootr::model::{
        DungeonReward,
        MainDungeon,
        Medallion,
        Stone,
    },
};

const DUNGEON_POSITIONS: [(MainDungeon, usize); 8] = [
    (MainDungeon::DekuTree, 0x1c),
    (MainDungeon::DodongosCavern, 0x1d),
    (MainDungeon::JabuJabu, 0x1e),
    (MainDungeon::ForestTemple, 0x1f),
    (MainDungeon::FireTemple, 0x20),
    (MainDungeon::WaterTemple, 0x21),
    (MainDungeon::SpiritTemple, 0x22),
    (MainDungeon::ShadowTemple, 0x23),
];

fn version_buf_len(version: u32) -> Option<usize> {
    Some(match version {
        0 => return None,
        1 => 0x38,
        _ => return None,
    })
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TrackerCtx {
    pub cfg_dungeon_info_enable: bool,
    pub cfg_dungeon_info_reward_enable: bool,
    pub cfg_dungeon_info_reward_need_compass: bool,
    pub cfg_dungeon_info_reward_need_altar: bool,
    pub cfg_dungeon_rewards: HashMap<MainDungeon, DungeonReward>,
}

impl TrackerCtx {
    pub fn new(data: &[u8]) -> Self {
        let version = BigEndian::read_u32(data);
        match version {
            0 => Self::default(),
            1 => Self {
                cfg_dungeon_info_enable: BigEndian::read_u32(&data[0x04..0x08]) != 0,
                cfg_dungeon_info_reward_enable: BigEndian::read_u32(&data[0x10..0x14]) != 0,
                cfg_dungeon_info_reward_need_compass: BigEndian::read_u32(&data[0x14..0x18]) != 0,
                cfg_dungeon_info_reward_need_altar: BigEndian::read_u32(&data[0x18..0x1c]) != 0,
                cfg_dungeon_rewards: {
                    let mut map = HashMap::with_capacity(8);
                    for (dungeon, pos) in &DUNGEON_POSITIONS {
                        map.insert(*dungeon, match data[*pos] {
                            0 => DungeonReward::Stone(Stone::KokiriEmerald),
                            1 => DungeonReward::Stone(Stone::GoronRuby),
                            2 => DungeonReward::Stone(Stone::ZoraSapphire),
                            3 => DungeonReward::Medallion(Medallion::Forest),
                            4 => DungeonReward::Medallion(Medallion::Fire),
                            5 => DungeonReward::Medallion(Medallion::Water),
                            6 => DungeonReward::Medallion(Medallion::Spirit),
                            7 => DungeonReward::Medallion(Medallion::Shadow),
                            8 => DungeonReward::Medallion(Medallion::Light),
                            n => unimplemented!("unknown boss reward index: {}", n),
                        });
                    }
                    map
                },
            },
            _ => unimplemented!("auto-tracker context version {} not supported", version),
        }
    }

    fn serialize(&self) -> Vec<u8> {
        let TrackerCtx { cfg_dungeon_info_enable, cfg_dungeon_info_reward_enable, cfg_dungeon_info_reward_need_compass, cfg_dungeon_info_reward_need_altar, ref cfg_dungeon_rewards } = *self;
        let current_version = 1;
        let mut buf = vec![0; version_buf_len(current_version).expect("missing auto-tracker context length for current version")];
        buf.splice(0x00..0x04, current_version.to_be_bytes().into_iter());
        buf.splice(0x04..0x08, if cfg_dungeon_info_enable { 1u32 } else { 0 }.to_be_bytes().into_iter());
        buf.splice(0x10..0x14, if cfg_dungeon_info_reward_enable { 1u32 } else { 0 }.to_be_bytes().into_iter());
        buf.splice(0x14..0x18, if cfg_dungeon_info_reward_need_compass { 1u32 } else { 0 }.to_be_bytes().into_iter());
        buf.splice(0x18..0x1c, if cfg_dungeon_info_reward_need_altar { 1u32 } else { 0 }.to_be_bytes().into_iter());
        for (dungeon, pos) in &DUNGEON_POSITIONS {
            buf[*pos] = if let Some(reward) = cfg_dungeon_rewards.get(dungeon) {
                match reward {
                    DungeonReward::Stone(Stone::KokiriEmerald) => 0,
                    DungeonReward::Stone(Stone::GoronRuby) => 1,
                    DungeonReward::Stone(Stone::ZoraSapphire) => 2,
                    DungeonReward::Medallion(Medallion::Forest) => 3,
                    DungeonReward::Medallion(Medallion::Fire) => 4,
                    DungeonReward::Medallion(Medallion::Water) => 5,
                    DungeonReward::Medallion(Medallion::Spirit) => 6,
                    DungeonReward::Medallion(Medallion::Shadow) => 7,
                    DungeonReward::Medallion(Medallion::Light) => 8,
                }
            } else {
                0xff
            };
        }
        buf
    }
}

impl Default for TrackerCtx {
    fn default() -> Self {
        Self {
            cfg_dungeon_info_enable: false,
            cfg_dungeon_info_reward_enable: false,
            cfg_dungeon_info_reward_need_compass: true,
            cfg_dungeon_info_reward_need_altar: true,
            cfg_dungeon_rewards: HashMap::default(),
        }
    }
}

impl Protocol for TrackerCtx {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let version = u32::read(stream).await?;
            Ok(if let Some(len) = version_buf_len(version) {
                let mut buf = vec![0; 4 + len];
                buf.splice(0..4, version.to_be_bytes().into_iter());
                stream.read_exact(&mut buf[4..]).await?;
                Self::new(&buf)
            } else if version == 0 {
                Self::default()
            } else {
                return Err(ReadError::UnknownVariant32(version))
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            sink.write_all(&self.serialize()).await?;
            Ok(())
        })
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        sink.write_all(&self.serialize())?;
        Ok(())
    }
}
