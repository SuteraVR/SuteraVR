use messaging::version::Version;

pub mod clocking;
pub mod messaging;
pub mod util;

pub struct Foo {}

pub const SCHEMA_VERSION: Version = Version {
    major: 0,
    minor: 1,
    patch: 0,
};
