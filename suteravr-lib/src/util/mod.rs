pub mod logger;

use alkahest::{serialize_to_vec, Formula, Serialize};
use enum_map::{EnumArray, EnumMap};

#[inline]
pub(crate) fn search_from_enum<T: EnumArray<U>, U: PartialEq>(
    mapping: EnumMap<T, U>,
    value: &U,
) -> Option<T> {
    mapping.iter().find(|(_, v)| *v == value).map(|(k, _)| k)
}

#[inline]
pub fn serialize_to_new_vec<T: Formula + Serialize<T>>(payload: T) -> Vec<u8> {
    let mut data = Vec::<u8>::new();
    let (size, _) = serialize_to_vec::<T, T>(payload, &mut data);
    data.truncate(size);
    data
}
