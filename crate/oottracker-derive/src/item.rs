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
    pyo3::prelude::*,
    quote::quote,
    syn::Ident,
};

pub(crate) fn to_ident(name: &str) -> Ident {
    Ident::new(&name.replace(&['\'', '(', ')', '[', ']'][..], "").to_case(Case::Pascal), Span::call_site())
}

pub(crate) fn item(item_list: &PyModule) -> Result<TokenStream, crate::Error> {
    let items = item_list.getattr("item_table")?
        .call_method0("items")?
        .iter()?
        .map(|elt| {
            let (name, (kind, _, _, _)) = elt?.extract::<(String, (String, &PyAny, &PyAny, &PyAny))>()?;
            PyResult::Ok((name, kind))
        })
        .try_collect::<_, Vec<_>, _>()?
        .into_iter()
        .filter_map(|(name, kind)| if kind != "Event" || name == "Scarecrow Song" { //HACK treat Scarecrow Song as not an event since it's not defined as one in any region
            Some(to_ident(&name))
        } else {
            None
        })
        .collect_vec();
    Ok(quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Protocol)]
        pub enum Item {
            #(#items,)*
        }
    })
}
