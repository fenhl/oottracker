use {
    std::collections::BTreeMap,
    serde::Deserialize,
    ootr::{
        model::Dungeon,
        region::Mq,
    },
    crate::RandoErr,
};

#[derive(Deserialize)]
pub(crate) struct RawRegion {
    pub region_name: String,
    #[allow(unused)] // taken from filename
    dungeon: Option<String>,
    pub scene: Option<String>,
    pub hint: Option<String>,
    #[serde(default)]
    pub time_passes: bool,
    #[serde(default)]
    pub events: BTreeMap<String, String>,
    #[serde(default)]
    pub locations: BTreeMap<String, String>,
    #[serde(default)]
    pub exits: BTreeMap<String, String>,
}

pub(crate) fn parse_dungeon_info(mut s: &str) -> Result<Option<(Dungeon, Mq)>, RandoErr> {
    Ok(if s == "Overworld" {
        None
    } else {
        let mq = if let Some(prefix) = s.strip_suffix(" MQ") {
            s = prefix;
            Mq::Mq
        } else {
            Mq::Vanilla
        };
        Some((s.parse().map_err(|()| RandoErr::UnknownRegionFilename(s.to_owned()))?, mq))
    })
}
