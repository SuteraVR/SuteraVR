//! `0.1.0` のようなセマンティックバージョンを表す構造体を提供するモジュール

use thiserror::Error;

/// `0.1.0` のようなセマンティックバージョンを表す構造体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Semver {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
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

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SemverSerdeError {
    #[error("Length of bytes must be 3, but got {0}")]
    UnexpectedLength(usize),
}

impl TryFrom<&[u8]> for Semver {
    type Error = SemverSerdeError;

    /// バージョンを表す`u8; 3` のバイナリを[`Semver`]に変換します。
    ///
    /// ```
    /// use suteravr_lib::semver::Semver;
    ///
    /// let some_binary: Vec<u8> = vec![0, 0, 1, 2, 3, 0, 0];
    /// let version: Semver = some_binary[2..5].try_into().unwrap();
    ///
    /// assert_eq!(
    ///   version,
    ///   Semver {
    ///     major: 1,
    ///     minor: 2,
    ///     patch: 3,
    ///   }
    /// );
    /// ```
    ///
    /// # Errors
    /// バイト列の長さが3でない場合は[`SemverSerdeError::UnexpectedLength`]が返されます。
    /// ```
    /// use suteravr_lib::semver::{Semver, SemverSerdeError};
    ///
    /// let some_binary: Vec<u8> = vec![0, 0, 1, 2, 3, 0, 0];
    /// let version: Result<Semver, SemverSerdeError> = some_binary[2..6].try_into();
    ///
    /// assert_eq!(
    ///   version,
    ///   Err(SemverSerdeError::UnexpectedLength(4))
    /// );
    /// ```
    ///
    ///
    fn try_from(val: &[u8]) -> Result<Self, Self::Error> {
        if val.len() != 3 {
            Err(SemverSerdeError::UnexpectedLength(val.len()))
        } else {
            Ok(Self {
                major: val[0],
                minor: val[1],
                patch: val[2],
            })
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
    /// let version: Semver = "1.12.2" .into();
    /// version.try_write(&mut buf[1..4]).unwrap();
    ///
    /// assert_eq!(
    ///   buf,
    ///   vec![0, 1, 12, 2, 0, 0]
    /// );
    /// ```
    ///
    /// # Errors
    /// バイト列の長さが3でない場合は[`SemverSerdeError::UnexpectedLength`]が返されます。
    /// ```
    /// use suteravr_lib::semver::{Semver, SemverSerdeError};
    ///
    /// let mut buf = vec![0, 0, 0, 0, 0, 0];
    /// let version: Semver = "1.12.2" .into();
    /// let result = version.try_write(&mut buf[1..5]);
    /// assert_eq!(
    ///  result,
    ///  Err(SemverSerdeError::UnexpectedLength(4))
    /// );
    /// ```
    ///
    ///
    pub fn try_write(&self, buf: &mut [u8]) -> Result<(), SemverSerdeError> {
        if buf.len() != 3 {
            Err(SemverSerdeError::UnexpectedLength(buf.len()))
        } else {
            buf[0] = self.major;
            buf[1] = self.minor;
            buf[2] = self.patch;
            Ok(())
        }
    }
}
