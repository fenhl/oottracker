use {
    std::{
        io,
        sync::Arc,
    },
    smart_default::SmartDefault,
};
#[cfg(not(target_arch = "wasm32"))] use {
    async_trait::async_trait,
    tokio::net::TcpStream,
    crate::proto::Protocol,
};

#[derive(Debug, SmartDefault, Clone, Copy)]
pub enum DungeonRewardLocation {
    #[default]
    Unknown,
    DekuTree,
    DodongosCavern,
    JabuJabu,
    ForestTemple,
    FireTemple,
    WaterTemple,
    ShadowTemple,
    SpiritTemple,
    LinksPocket,
}

#[derive(Debug, Clone)]
pub enum DungeonRewardLocationReadError {
    Io(Arc<io::Error>),
    UnknownLocationId(u8),
}

impl From<io::Error> for DungeonRewardLocationReadError {
    fn from(e: io::Error) -> DungeonRewardLocationReadError {
        DungeonRewardLocationReadError::Io(Arc::new(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl Protocol for DungeonRewardLocation {
    type ReadError = DungeonRewardLocationReadError;

    async fn read(tcp_stream: &mut TcpStream) -> Result<DungeonRewardLocation, DungeonRewardLocationReadError> {
        match u8::read(tcp_stream).await? {
            0 => Ok(DungeonRewardLocation::Unknown),
            1 => Ok(DungeonRewardLocation::DekuTree),
            2 => Ok(DungeonRewardLocation::DodongosCavern),
            3 => Ok(DungeonRewardLocation::JabuJabu),
            4 => Ok(DungeonRewardLocation::ForestTemple),
            5 => Ok(DungeonRewardLocation::FireTemple),
            6 => Ok(DungeonRewardLocation::WaterTemple),
            7 => Ok(DungeonRewardLocation::ShadowTemple),
            8 => Ok(DungeonRewardLocation::SpiritTemple),
            9 => Ok(DungeonRewardLocation::LinksPocket),
            n => Err(DungeonRewardLocationReadError::UnknownLocationId(n)),
        }
    }

    async fn write(&self, tcp_stream: &mut TcpStream) -> io::Result<()> {
        match self {
            DungeonRewardLocation::Unknown => 0u8.write(tcp_stream).await,
            DungeonRewardLocation::DekuTree => 1u8.write(tcp_stream).await,
            DungeonRewardLocation::DodongosCavern => 2u8.write(tcp_stream).await,
            DungeonRewardLocation::JabuJabu => 3u8.write(tcp_stream).await,
            DungeonRewardLocation::ForestTemple => 4u8.write(tcp_stream).await,
            DungeonRewardLocation::FireTemple => 5u8.write(tcp_stream).await,
            DungeonRewardLocation::WaterTemple => 6u8.write(tcp_stream).await,
            DungeonRewardLocation::ShadowTemple => 7u8.write(tcp_stream).await,
            DungeonRewardLocation::SpiritTemple => 8u8.write(tcp_stream).await,
            DungeonRewardLocation::LinksPocket => 9u8.write(tcp_stream).await,
        }
    }

    fn write_sync(&self, tcp_stream: &mut std::net::TcpStream) -> io::Result<()> {
        match self {
            DungeonRewardLocation::Unknown => 0u8.write_sync(tcp_stream),
            DungeonRewardLocation::DekuTree => 1u8.write_sync(tcp_stream),
            DungeonRewardLocation::DodongosCavern => 2u8.write_sync(tcp_stream),
            DungeonRewardLocation::JabuJabu => 3u8.write_sync(tcp_stream),
            DungeonRewardLocation::ForestTemple => 4u8.write_sync(tcp_stream),
            DungeonRewardLocation::FireTemple => 5u8.write_sync(tcp_stream),
            DungeonRewardLocation::WaterTemple => 6u8.write_sync(tcp_stream),
            DungeonRewardLocation::ShadowTemple => 7u8.write_sync(tcp_stream),
            DungeonRewardLocation::SpiritTemple => 8u8.write_sync(tcp_stream),
            DungeonRewardLocation::LinksPocket => 9u8.write_sync(tcp_stream),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Knowledge {
    pub kokiri_emerald_location: DungeonRewardLocation,
    pub goron_ruby_location: DungeonRewardLocation,
    pub zora_sapphire_location: DungeonRewardLocation,
    pub forest_medallion_location: DungeonRewardLocation,
    pub fire_medallion_location: DungeonRewardLocation,
    pub water_medallion_location: DungeonRewardLocation,
    pub shadow_medallion_location: DungeonRewardLocation,
    pub spirit_medallion_location: DungeonRewardLocation,
    pub light_medallion_location: DungeonRewardLocation,
}

impl Knowledge {
    /// We know that everything is vanilla. Used by auto-trackers when the base game, rather than rando, is detected.
    pub fn vanilla() -> Knowledge {
        Knowledge {
            kokiri_emerald_location: DungeonRewardLocation::DekuTree,
            goron_ruby_location: DungeonRewardLocation::DodongosCavern,
            zora_sapphire_location: DungeonRewardLocation::JabuJabu,
            forest_medallion_location: DungeonRewardLocation::ForestTemple,
            fire_medallion_location: DungeonRewardLocation::FireTemple,
            water_medallion_location: DungeonRewardLocation::WaterTemple,
            shadow_medallion_location: DungeonRewardLocation::ShadowTemple,
            spirit_medallion_location: DungeonRewardLocation::SpiritTemple,
            light_medallion_location: DungeonRewardLocation::LinksPocket,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl Protocol for Knowledge {
    type ReadError = DungeonRewardLocationReadError;

    async fn read(tcp_stream: &mut TcpStream) -> Result<Knowledge, DungeonRewardLocationReadError> {
        Ok(Knowledge {
            kokiri_emerald_location: DungeonRewardLocation::read(tcp_stream).await?,
            goron_ruby_location: DungeonRewardLocation::read(tcp_stream).await?,
            zora_sapphire_location: DungeonRewardLocation::read(tcp_stream).await?,
            forest_medallion_location: DungeonRewardLocation::read(tcp_stream).await?,
            fire_medallion_location: DungeonRewardLocation::read(tcp_stream).await?,
            water_medallion_location: DungeonRewardLocation::read(tcp_stream).await?,
            shadow_medallion_location: DungeonRewardLocation::read(tcp_stream).await?,
            spirit_medallion_location: DungeonRewardLocation::read(tcp_stream).await?,
            light_medallion_location: DungeonRewardLocation::read(tcp_stream).await?,
        })
    }

    async fn write(&self, tcp_stream: &mut TcpStream) -> io::Result<()> {
        self.kokiri_emerald_location.write(tcp_stream).await?;
        self.goron_ruby_location.write(tcp_stream).await?;
        self.zora_sapphire_location.write(tcp_stream).await?;
        self.forest_medallion_location.write(tcp_stream).await?;
        self.fire_medallion_location.write(tcp_stream).await?;
        self.water_medallion_location.write(tcp_stream).await?;
        self.shadow_medallion_location.write(tcp_stream).await?;
        self.spirit_medallion_location.write(tcp_stream).await?;
        self.light_medallion_location.write(tcp_stream).await?;
        Ok(())
    }

    fn write_sync(&self, tcp_stream: &mut std::net::TcpStream) -> io::Result<()> {
        self.kokiri_emerald_location.write_sync(tcp_stream)?;
        self.goron_ruby_location.write_sync(tcp_stream)?;
        self.zora_sapphire_location.write_sync(tcp_stream)?;
        self.forest_medallion_location.write_sync(tcp_stream)?;
        self.fire_medallion_location.write_sync(tcp_stream)?;
        self.water_medallion_location.write_sync(tcp_stream)?;
        self.shadow_medallion_location.write_sync(tcp_stream)?;
        self.spirit_medallion_location.write_sync(tcp_stream)?;
        self.light_medallion_location.write_sync(tcp_stream)?;
        Ok(())
    }
}
