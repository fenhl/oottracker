use {
    std::fmt,
    num_traits::One,
};

pub(crate) fn plural(n: impl PartialEq + One + fmt::Display, singular: impl fmt::Display) -> String {
    format!("{} {}{}", n, singular, if n == One::one() { "" } else { "s" })
}
