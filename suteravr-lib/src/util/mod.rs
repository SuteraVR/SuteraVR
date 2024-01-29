use enum_map::{EnumArray, EnumMap};

#[inline]
pub(crate) fn search_from_enum<T: EnumArray<U>, U: PartialEq>(
    mapping: EnumMap<T, U>,
    value: &U,
) -> Option<T> {
    mapping.iter().find(|(_, v)| *v == value).map(|(k, _)| k)
}
