use {
    convert_case::{
        Case,
        Casing as _,
    },
    itertools::Itertools as _,
    proc_macro2::{
        Span,
        TokenStream,
    },
    pyo3::{
        AsPyPointer as _,
        PyNativeType as _,
        prelude::*,
        types::{
            PyBool,
            PyDict,
            PyInt,
            PyList,
            PyString,
            PyType,
        },
    },
    quote::quote,
    syn::Ident,
    crate::Error,
};

fn ignore_setting(name: &str) -> bool {
    match name {
        "web_wad_file"
        | "web_common_key_file"
        | "web_common_key_string"
        | "web_wad_channel_id"
        | "web_wad_channel_title"
        | "web_output_type"
        | "web_persist_in_cache"
        | "cosmetics_only"
        | "check_version"
        | "output_settings"
        | "generate_from_file"
        | "enable_distribution_file"
        | "enable_cosmetic_file"
        | "distribution_file"
        | "cosmetic_file"
        | "checked_version"
        | "rom"
        | "output_dir"
        | "output_file"
        | "seed"
        | "patch_file"
        | "count"
        | "presets"
        | "open_output_dir"
        | "open_python_dir"
        | "repatch_cosmetics"
        | "create_spoiler"
        | "create_cosmetics_log"
        | "compress_rom"
        | "tricks_list_msg" => true, // ignore patching and GUI infrastructure
        "bingosync_url"
        | "item_hints" => true, // ignoring bingo stuff for now //TODO use for hints knowledge and routing goal?
        "hint_dist_user" => true, //TODO handle settings with no display name, handle hint_dist_user structure
        _ => false
    }
}

pub(crate) fn value_to_ident(value: &str) -> Ident {
    Ident::new(match value {
        "0" => "zero",
        "1" => "one",
        "2" => "two",
        "3" => "three",
        "4" => "four",
        "default" => "forenoon", //TODO make sure this is only on starting time of day
        "witching-hour" => "witching_hour",
        _ => value,
    }, Span::call_site())
}

pub(crate) fn settings(settings_list: &PyModule) -> Result<TokenStream, Error> {
    let mut knowledge_types = Vec::default();
    let mut knowledge_fields = Vec::default();
    let mut knowledge_defaults = Vec::default();
    let mut knowledge_bit_and = Vec::default();
    for setting in settings_list.getattr("setting_infos")?.iter()? {
        let setting = setting?;
        let name = setting.getattr("name")?.extract::<String>()?;
        if setting.getattr("cosmetic")?.extract()? { continue } // ignore cosmetic settings for now //TODO use to style items on the GUI?
        if ignore_setting(&name) { continue }
        let name_ident = Ident::new(&name, Span::call_site());
        let setting_type = setting.getattr("type")?;
        if settings_list.getattr("Combobox")?.downcast::<PyType>()?.is_instance(setting)? {
            let ty = Ident::new(&format!("{}Knowledge", name.to_case(Case::Pascal)), Span::call_site());
            let choices = setting.getattr("choice_list")?.iter()?
                .map(|choice_res| choice_res.and_then(|choice| Ok(value_to_ident(choice.extract()?))))
                .try_collect::<_, Vec<_>, _>()?;
            knowledge_types.push(quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Protocol)]
                pub struct #ty {
                    #(pub #choices: bool,)*
                }

                impl #ty {
                    fn empty() -> Self {
                        Self {
                            #(#choices: false,)*
                        }
                    }

                    #(
                        pub fn #choices() -> Self {
                            Self {
                                #choices: true,
                                ..Self::empty()
                            }
                        }
                    )*

                    pub fn is_known(&self) -> bool {
                        0 #(+ u8::from(self.#choices))* == 1
                    }
                }

                impl Default for #ty {
                    fn default() -> Self {
                        Self {
                            #(#choices: true,)*
                        }
                    }
                }

                impl BitAnd for #ty {
                    type Output = Result<Self, Contradiction>;

                    fn bitand(mut self, rhs: Self) -> Result<Self, Contradiction> {
                        #(
                            self.#choices &= rhs.#choices;
                        )*
                        if self == Self::empty() { Err(Contradiction) } else { Ok(self) }
                    }
                }
            });
            knowledge_fields.push(quote!(#name_ident: #ty));
            knowledge_defaults.push(quote!(#name_ident: #ty::default()));
            knowledge_bit_and.push(quote!(self.#name_ident = (self.#name_ident & rhs.#name_ident)?;));
        } else if setting_type.as_ptr() == settings_list.py().get_type::<PyBool>().as_ptr() {
            knowledge_fields.push(quote!(#name_ident: Option<bool>));
            knowledge_defaults.push(quote!(#name_ident: None));
            knowledge_bit_and.push(quote! {
                match (self.#name_ident, rhs.#name_ident) {
                    (Some(true), Some(false)) | (Some(false), Some(true)) => return Err(Contradiction),
                    (Some(_), None) => {},
                    (_, b) => self.#name_ident = b,
                }
            });
        } else if setting_type.as_ptr() == settings_list.py().get_type::<PyInt>().as_ptr() {
            knowledge_fields.push(quote!(#name_ident: RangeInclusive<u8>));
            let min = setting.getattr("gui_params")?.downcast::<PyDict>()?.get_item("min").ok_or(Error::MissingIntSettingBound)?.extract::<u8>()?;
            let max = setting.getattr("gui_params")?.downcast::<PyDict>()?.get_item("max").ok_or(Error::MissingIntSettingBound)?.extract::<u8>()?;
            knowledge_defaults.push(quote!(#name_ident: #min..=#max));
            knowledge_bit_and.push(quote! {
                self.#name_ident = *self.#name_ident.start().max(rhs.#name_ident.start())..=*self.#name_ident.end().min(rhs.#name_ident.end());
                if self.#name_ident.end() < self.#name_ident.start() { return Err(Contradiction) }
            });
        } else if setting_type.as_ptr() == settings_list.py().get_type::<PyString>().as_ptr() {
            return Err(Error::UnknownStringSetting(name))
        } else if setting_type.as_ptr() == settings_list.py().get_type::<PyList>().as_ptr() {
            knowledge_fields.push(quote!(#name_ident: HashMap<Cow<'static, str>, bool>));
            knowledge_defaults.push(quote!(#name_ident: HashMap::default()));
            knowledge_bit_and.push(quote! {
                for (key, val2) in rhs.#name_ident {
                    if self.#name_ident.get(&key).map_or(false, |&val1| val1 != val2) { return Err(Contradiction) }
                    self.#name_ident.insert(key, val2);
                }
            });
        } else if setting_type.as_ptr() == settings_list.py().get_type::<PyDict>().as_ptr() {
            unimplemented!() //TODO hint_dist_user
        } else {
            return Err(Error::UnknownSettingType(name))
        }
    }
    Ok(quote! {
        #(#knowledge_types)*

        #[derive(Debug, Clone, PartialEq, Eq, Protocol)]
        pub struct Knowledge {
            #(pub #knowledge_fields,)*
        }

        impl Default for Knowledge {
            fn default() -> Self {
                Self {
                    #(#knowledge_defaults,)*
                }
            }
        }

        impl BitAnd for Knowledge {
            type Output = Result<Self, Contradiction>;

            fn bitand(mut self, rhs: Self) -> Result<Self, Contradiction> {
                #(#knowledge_bit_and)*
                Ok(self)
            }
        }
    })
}
