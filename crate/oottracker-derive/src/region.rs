use {
    std::{
        collections::BTreeMap,
        fs::{
            self,
            File,
        },
        io::{
            self,
            BufReader,
            prelude::*,
        },
        path::Path,
    },
    convert_case::{
        Case,
        Casing as _,
    },
    itertools::Itertools as _,
    proc_macro2::{
        Span,
        TokenStream,
    },
    quote::quote,
    serde::{
        Deserialize,
        de::DeserializeOwned,
    },
    syn::Ident,
    crate::Error,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRegion {
    region_name: String,
    #[allow(unused)] // taken from filename
    dungeon: Option<String>,
    #[allow(unused)] //TODO (this is the ER scene, not the game scene)
    scene: Option<String>,
    #[allow(unused)] //TODO
    hint: Option<String>,
    #[allow(unused)] //TODO
    #[serde(default)]
    time_passes: bool,
    #[allow(unused)] //TODO
    #[serde(default)]
    events: BTreeMap<String, String>,
    #[allow(unused)] //TODO
    #[serde(default)]
    locations: BTreeMap<String, String>,
    #[allow(unused)] //TODO
    #[serde(default)]
    exits: BTreeMap<String, String>,
}

fn read_json_lenient_sync<R: BufRead, T: DeserializeOwned>(mut reader: R) -> io::Result<T> {
    let mut buf = String::default();
    let mut line_buf = String::default();
    while reader.read_line(&mut line_buf)? > 0 {
        buf.push_str(
            &line_buf.split('#')
                .next().expect("split always yields at least one element")
                .replace("\r", "")
                .replace('\n', " ")
        );
        line_buf.clear();
    }
    Ok(serde_json::from_str(&buf)?)
}

fn parse_dungeon_info(mut s: &str) -> Option<(String, bool)> {
    if s == "Overworld" {
        None
    } else {
        let mq = if let Some(prefix) = s.strip_suffix(" MQ") {
            s = prefix;
            true
        } else {
            false
        };
        Some((s.to_owned(), mq))
    }
}

pub(crate) fn region(rando_path: &Path) -> Result<TokenStream, Error> {
    let world_path = rando_path.join("data").join("World"); //TODO glitched support
    let mut regions = Vec::default();
    for region_path in fs::read_dir(world_path)? {
        let region_path = region_path?;
        let filename = region_path.file_name();
        let filename = filename.to_str().ok_or(Error::NonUnicodeRegionFilename)?;
        let dungeon = parse_dungeon_info(filename.strip_suffix(".json").ok_or_else(|| Error::NonJsonRegionFile(filename.to_owned()))?);
        let region_file = File::open(region_path.path())?;
        for raw_region in read_json_lenient_sync::<_, Vec<RawRegion>>(BufReader::new(region_file))? {
            //assert_eq!(dungeon.map(|(dungeon, _)| dungeon.to_string().replace('\'', "")), raw_region.dungeon);
            /*
            regions.push(Arc::new(Region {
                dungeon,
                scene: raw_region.scene,
                hint: raw_region.hint,
                time_passes: raw_region.time_passes,
                events: raw_region.events.into_iter().map(|(event_name, rule_str)| Ok::<_, RandoErr>((event_name.clone(), ootr::access::Expr::parse(self, &Check::Event(event_name), rule_str.trim())?))).try_collect()?,
                locations: raw_region.locations.into_iter().map(|(loc_name, rule_str)| Ok::<_, RandoErr>((loc_name.clone(), ootr::access::Expr::parse(self, &Check::Location(loc_name), rule_str.trim())?))).try_collect()?,
                exits: raw_region.exits.into_iter().map(|(to, rule_str)| Ok::<_, RandoErr>((to.clone(), ootr::access::Expr::parse(self, &Check::Exit { to, from: name.clone(), from_mq: dungeon.map(|(_, mq)| mq) }, rule_str.trim())?))).try_collect()?,
                name,
            }));
            */
            regions.push((dungeon.clone(), raw_region));
        }
    }
    let (region_variants, region_names) = regions.into_iter()
        .map(|(dungeon, raw_region)| (
            Ident::new(&format!("{}{}", if let Some((_, true)) = dungeon { "Mq" } else { "" }, raw_region.region_name.to_case(Case::Pascal)), Span::call_site()),
            raw_region.region_name,
        ))
        .multiunzip::<(Vec<_>, Vec<_>)>();
    Ok(quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator, Protocol)]
        pub enum Region {
            #(#region_variants,)*
        }

        impl fmt::Display for Region { //TODO include full region info (glitched, MQ), add separate `name` method?
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match *self {
                    #(Self::#region_variants => #region_names.fmt(f),)*
                }
            }
        }
    })
}
