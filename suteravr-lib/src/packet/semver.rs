//! `0.1.0` のようなセマンティックバージョンを表す構造体を提供するモジュール

use crate::typing::SizedForBinary;

/// `0.1.0` のようなセマンティックバージョンを表す構造体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Semver {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl SizedForBinary for Semver {
    const SIZE: usize = 3;
}

impl From<&str> for Semver {
    /// バージョン文字列を[`Semver`]に変換します。
    ///
    /// ```
    /// use suteravr_lib::semver::Semver;
    ///
    /// let version: Semver = "0.1.0".into();
    ///
    /// assert_eq!(
    ///   version,
    ///   Semver {
    ///     major: 0,
    ///     minor: 1,
    ///     patch: 0,
    ///   }
    /// );
    /// ```
    fn from(version: &str) -> Self {
        let mut iter = version.split('.');
        let major = iter.next().unwrap().parse().unwrap();
        let minor = iter.next().unwrap().parse().unwrap();
        let patch = iter.next().unwrap().parse().unwrap();
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl From<Semver> for String {
    /// [`Semver`]をバージョン文字列に変換します。
    ///
    /// ```
    /// use suteravr_lib::semver::Semver;
    ///
    /// let version = Semver {
    ///   major: 0,
    ///   minor: 1,
    ///   patch: 0,
    /// };
    ///
    /// assert_eq!(String::from(version), "0.1.0");
    /// ```
    fn from(val: Semver) -> Self {
        format!("{}.{}.{}", val.major, val.minor, val.patch)
    }
}

impl From<[u8; 3]> for Semver {
    /// バージョンを表す`u8; 3` のバイナリを[`Semver`]に変換します。
    ///
    /// ```
    /// use suteravr_lib::semver::Semver;
    ///
    /// let some_binary: Vec<u8> = vec![0, 0, 1, 2, 3, 0, 0];
    /// let version_binary: [u8; 3] = some_binary[2..5].try_into().unwrap();
    /// let version: Semver = version_binary.into();
    /// assert_eq!(
    ///   version,
    ///   Semver {
    ///     major: 1,
    ///     minor: 2,
    ///     patch: 3,
    ///   }
    /// );
    /// ```
    fn from(val: [u8; 3]) -> Self {
        Self {
            major: val[0],
            minor: val[1],
            patch: val[2],
        }
    }
}

impl Semver {
    /// [`Semver`]をバージョンを表す`u8;3` のバイナリに変換します。
    ///
    /// ```
    /// use suteravr_lib::semver::Semver;
    ///
    /// let mut buf = vec![0, 0, 0, 0, 0, 0];
    /// let version: Semver = "1.12.2".into();
    /// let write_place: &mut [u8; 3] = (&mut buf[1..4]).try_into().unwrap();
    /// version.write(write_place);
    ///
    /// assert_eq!(
    ///   buf,
    ///   vec![0, 1, 12, 2, 0, 0]
    /// );
    /// ```
    ///
    ///
    pub fn write(&self, buf: &mut [u8; 3]) {
        buf[0] = self.major;
        buf[1] = self.minor;
        buf[2] = self.patch;
    }
}
