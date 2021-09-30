use {
    std::{
        collections::{
            BTreeMap,
            HashMap,
            HashSet,
        },
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
    pyo3::prelude::*,
    quote::quote,
    serde::{
        Deserialize,
        de::DeserializeOwned,
    },
    syn::Ident,
    crate::{
        Error,
        access,
    },
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
    #[serde(default)]
    events: BTreeMap<String, String>,
    #[allow(unused)] //TODO
    #[serde(default)]
    locations: BTreeMap<String, String>,
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

pub(crate) fn region<'p>(py: Python<'p>, rando_path: &Path) -> Result<TokenStream, Error> {
    let sys = py.import("sys")?;
    sys.getattr("path")?.call_method1("append", (rando_path.display().to_string(),))?;
    // load region files
    let world_path = rando_path.join("data").join("World"); //TODO glitched support
    let mut events = HashSet::default();
    let mut regions = Vec::default();
    for region_path in fs::read_dir(world_path)? {
        let region_path = region_path?;
        let filename = region_path.file_name();
        let filename = filename.to_str().ok_or(Error::NonUnicodeRegionFilename)?;
        let dungeon = parse_dungeon_info(filename.strip_suffix(".json").ok_or_else(|| Error::NonJsonRegionFile(filename.to_owned()))?);
        let region_file = File::open(region_path.path())?;
        for raw_region in read_json_lenient_sync::<_, Vec<RawRegion>>(BufReader::new(region_file))? {
            //assert_eq!(dungeon.map(|(dungeon, _)| dungeon.to_string().replace('\'', "")), raw_region.dungeon);
            for (event_name, _) in &raw_region.events {
                events.insert(event_name.clone());
            }
            regions.push((dungeon.clone(), raw_region));
        }
    }
    // parse logic helpers
    let raw_helpers = read_json_lenient_sync::<_, BTreeMap<String, String>>(BufReader::new(File::open(rando_path.join("data").join("LogicHelpers.json"))?))?;
    let mut helper_headers = HashMap::new();
    for (fn_def, fn_body) in &raw_helpers {
        let (fn_name, fn_params) = if fn_def.contains('(') {
            fn_def[..fn_def.len() - 1].split('(').collect_tuple().ok_or(Error::InvalidLogicHelper)?
        } else {
            (&**fn_def, "")
        };
        let fn_params = if fn_params.is_empty() {
            Vec::default()
        } else {
            fn_params.split(',').map(str::to_owned).collect_vec()
        };
        helper_headers.insert(fn_name.to_owned(), (fn_params, fn_body));
    }
    let arities = helper_headers.iter().map(|(fn_name, (fn_params, _))| (&**fn_name, fn_params.len())).collect();
    let mut helpers = HashMap::new();
    for (fn_name, (fn_params, fn_body)) in &helper_headers {
        let ctx = access::Context::LogicHelper(fn_name.to_owned());
        let expr = access::Expr::parse_helper(py, &ctx, &arities, &events, &fn_params, &fn_body)?;
        helpers.insert(fn_name.to_owned(), (fn_params.clone(), expr));
    }
    // generate regions code
    let mut region_names = Vec::default();
    let mut region_variants = Vec::default();
    let mut exits_arms = Vec::default();
    for (dungeon, raw_region) in &regions {
        let name = &raw_region.region_name;
        let variant = Ident::new(&format!("{}{}", if let Some((_, true)) = dungeon { "Mq" } else { "" }, name.to_case(Case::Pascal)), Span::call_site());
        let mut insert_exits = Vec::default();
        for (target_name, access) in &raw_region.exits {
            match &*regions.iter().filter(|(iter_dungeon, iter_region)| dungeon.as_ref().map_or(true, |(_, mq)| iter_dungeon.as_ref().map_or(true, |(_, iter_mq)| mq == iter_mq)) && iter_region.region_name == *target_name).collect_vec() {
                [] => return Err(Error::ExitToUnknownRegion),
                [(target_dungeon, _)] => {
                    let target_variant = Ident::new(&format!("{}{}", if let Some((_, true)) = target_dungeon { "Mq" } else { "" }, target_name.to_case(Case::Pascal)), Span::call_site());
                    let target_access_fn = Ident::new(&format!("{}{}_access", if let Some((_, true)) = target_dungeon { "mq_" } else { "" }, target_name.to_case(Case::Snake)), Span::call_site());
                    let target_access_expr = access::Expr::parse(py, &access::Context::Exit { from: name.clone(), to: target_name.clone() }, &helpers, &events, access)?.compile();
                    insert_exits.push(quote! {
                        fn #target_access_fn(#[allow(unused)] model: &ModelState) -> bool { #target_access_expr }

                        map.insert(Self::#target_variant, #target_access_fn);
                    });
                }
                [(Some((target_dungeon, false)), _ /*vanilla_target*/), (Some((_, true)), _ /*mq_target*/)] | [(Some((_, true)), _ /*mq_target*/), (Some((target_dungeon, false)), _ /*vanilla_target*/)] => {
                    //TODO delay dismbiguator for dungeons where it can't be checked in the first region (see comments on Ram::current_region for details)
                    let disambiguator_variant = Ident::new(&format!("{}MqDisambig", target_dungeon.to_case(Case::Pascal)), Span::call_site());
                    let vanilla_variant = Ident::new(&target_name.to_case(Case::Pascal), Span::call_site());
                    let mq_variant = Ident::new(&format!("Mq{}", target_name.to_case(Case::Pascal)), Span::call_site());
                    let disambiguator_access_fn = Ident::new(&format!("{}_access", target_dungeon.to_case(Case::Snake)), Span::call_site());
                    let vanilla_access = Ident::new(&format!("{}_access", target_name.to_case(Case::Snake)), Span::call_site());
                    let mq_access = Ident::new(&format!("{}_mq_access", target_name.to_case(Case::Snake)), Span::call_site());
                    let disambiguator_access_expr = access::Expr::parse(py, &access::Context::Exit { from: name.clone(), to: target_name.clone() }, &helpers, &events, access)?.compile();
                    insert_exits.push(quote! {
                        fn #disambiguator_access_fn(#[allow(unused)] model: &ModelState) -> bool { #disambiguator_access_expr }

                        map.insert(Self::#disambiguator_variant, #disambiguator_access_fn);
                    });
                    let dungeon = match &**target_dungeon {
                        "Deku Tree" => quote!(Dungeon::Main(MainDungeon::DekuTree)),
                        "Dodongos Cavern" => quote!(Dungeon::Main(MainDungeon::DodongosCavern)),
                        "Jabu Jabus Belly" => quote!(Dungeon::Main(MainDungeon::JabuJabu)),
                        "Forest Temple" => quote!(Dungeon::Main(MainDungeon::ForestTemple)),
                        "Fire Temple" => quote!(Dungeon::Main(MainDungeon::FireTemple)),
                        "Water Temple" => quote!(Dungeon::Main(MainDungeon::WaterTemple)),
                        "Shadow Temple" => quote!(Dungeon::Main(MainDungeon::ShadowTemple)),
                        "Spirit Temple" => quote!(Dungeon::Main(MainDungeon::SpiritTemple)),
                        "Ice Cavern" => quote!(Dungeon::IceCavern),
                        "Bottom of the Well" => quote!(Dungeon::BottomOfTheWell),
                        "Gerudo Training Ground" => quote!(Dungeon::GerudoTrainingGround),
                        "Ganons Castle" => quote!(Dungeon::GanonsCastle),
                        _ => return Err(Error::UnknownDungeon(target_dungeon.to_owned())),
                    };
                    exits_arms.push(quote! {
                        Self::#disambiguator_variant => {
                            fn #vanilla_access(model: &ModelState) -> bool {
                                model.knowledge.dungeons.get(&#dungeon).map_or(false, |&mq| mq == Mq::Vanilla)
                            }

                            fn #mq_access(model: &ModelState) -> bool {
                                model.knowledge.dungeons.get(&#dungeon).map_or(false, |&mq| mq == Mq::Mq)
                            }

                            map.insert(Self::#vanilla_variant, #vanilla_access);
                            map.insert(Self::#mq_variant, #mq_access);
                        }
                    });
                }
                _ => panic!("multiple regions with the target name found"),
            }
        }
        exits_arms.push(quote!(Self::#variant => { #(#insert_exits)* }));
        region_names.push(name.clone());
        region_variants.push(variant);
    }
    Ok(quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator, Protocol)] //TODO stable Protocol representation, e.g. by full region info string as in the todo comment on Display?
        pub enum Region {
            DekuTreeMqDisambig,
            DodongosCavernMqDisambig,
            JabuJabusBellyMqDisambig,
            ForestTempleMqDisambig,
            FireTempleMqDisambig,
            WaterTempleMqDisambig,
            ShadowTempleMqDisambig,
            SpiritTempleMqDisambig,
            IceCavernMqDisambig,
            BottomOfTheWellMqDisambig,
            GerudoTrainingGroundMqDisambig,
            GanonsCastleMqDisambig,
            #(#region_variants,)*
        }

        impl Region {
            pub fn exits(&self) -> HashMap<Region, Access> {
                let mut map = HashMap::<_, Access>::default();
                match self {
                    #(#exits_arms,)*
                }
                map
            }
        }

        impl fmt::Display for Region { //TODO include full region info (glitched, MQ), add separate `name` method?
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    Self::DekuTreeMqDisambig => write!(f, "Deku Tree (possibly MQ)"),
                    Self::DodongosCavernMqDisambig => write!(f, "Dodongo's Cavern (possibly MQ)"),
                    Self::JabuJabusBellyMqDisambig => write!(f, "Jabu Jabu's Belly (possibly MQ)"),
                    Self::ForestTempleMqDisambig => write!(f, "Forest Temple (possibly MQ)"),
                    Self::FireTempleMqDisambig => write!(f, "Fire Temple (possibly MQ)"),
                    Self::WaterTempleMqDisambig => write!(f, "Water Temple (possibly MQ)"),
                    Self::ShadowTempleMqDisambig => write!(f, "Shadow Temple (possibly MQ)"),
                    Self::SpiritTempleMqDisambig => write!(f, "Spirit Temple (possibly MQ)"),
                    Self::IceCavernMqDisambig => write!(f, "Ice Cavern (possibly MQ)"),
                    Self::BottomOfTheWellMqDisambig => write!(f, "Bottom of the Well (possibly MQ)"),
                    Self::GerudoTrainingGroundMqDisambig => write!(f, "Gerudo Training Ground (possibly MQ)"),
                    Self::GanonsCastleMqDisambig => write!(f, "inside Ganon's Castle (possibly MQ)"),
                    #(Self::#region_variants => #region_names.fmt(f),)*
                }
            }
        }
    })
}
