// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::io::Error as IoError;
use std::result::Result;

use libc::{uname, utsname};

#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum KernelVersionError {
    /// Error calling uname: {0}
    Uname(#[from] IoError),
    /// Invalid utf-8: {0}
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    /// Invalid kernel version format
    InvalidFormat,
    /// Invalid integer: {0}
    InvalidInt(#[from] std::num::ParseIntError),
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct KernelVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

impl KernelVersion {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn get() -> Result<Self, KernelVersionError> {
        let mut name: utsname = utsname {
            sysname: [0; 65],
            nodename: [0; 65],
            release: [0; 65],
            version: [0; 65],
            machine: [0; 65],
            domainname: [0; 65],
        };
        // SAFETY: Safe because the parameters are valid.
        let res = unsafe { uname((&mut name) as *mut utsname) };

        if res < 0 {
            return Err(KernelVersionError::Uname(IoError::last_os_error()));
        }

        Self::parse(String::from_utf8(
            #[allow(clippy::useless_conversion)]
            name.release
                .iter()
                .map(|c| u8::try_from(*c).unwrap())
                .collect(),
        )?)
    }

    fn parse(release: String) -> Result<Self, KernelVersionError> {
        let mut tokens = release.split('.');

        let major = tokens.next().ok_or(KernelVersionError::InvalidFormat)?;
        let minor = tokens.next().ok_or(KernelVersionError::InvalidFormat)?;
        let mut patch = tokens.next().ok_or(KernelVersionError::InvalidFormat)?;

        // Parse the `patch`, since it may contain other tokens as well.
        if let Some(index) = patch.find(|c: char| !c.is_ascii_digit()) {
            patch = &patch[..index];
        }

        Ok(Self {
            major: major.parse()?,
            minor: minor.parse()?,
            patch: patch.parse()?,
        })
    }
}

impl std::fmt::Display for KernelVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get() {
        KernelVersion::get().unwrap();
    }

    #[test]
    fn test_parse_valid() {
        assert_eq!(
            KernelVersion::parse("5.10.0".to_string()).unwrap(),
            KernelVersion::new(5, 10, 0),
        );
        assert_eq!(
            KernelVersion::parse("5.10.50".to_string()).unwrap(),
            KernelVersion::new(5, 10, 50)
        );
        assert_eq!(
            KernelVersion::parse("5.10.50-38.132.amzn2int.x86_64".to_string()).unwrap(),
            KernelVersion::new(5, 10, 50)
        );
    }

    #[test]
    fn test_parse_invalid() {
        KernelVersion::parse("".to_string()).unwrap_err();
        KernelVersion::parse("ffff".to_string()).unwrap_err();
        KernelVersion::parse("ffff.55.0".to_string()).unwrap_err();
        KernelVersion::parse("5.10.".to_string()).unwrap_err();
        KernelVersion::parse("5.0".to_string()).unwrap_err();
        KernelVersion::parse("5.0fff".to_string()).unwrap_err();
    }

    #[test]
    fn test_cmp() {
        // Comparing major.
        assert!(KernelVersion::new(4, 0, 0) < KernelVersion::new(5, 10, 15));
        assert!(KernelVersion::new(4, 0, 0) > KernelVersion::new(3, 10, 15));

        // Comparing minor.
        assert!(KernelVersion::new(5, 0, 20) < KernelVersion::new(5, 10, 15));
        assert!(KernelVersion::new(5, 20, 20) > KernelVersion::new(5, 10, 15));
        assert!(KernelVersion::new(5, 100, 20) > KernelVersion::new(5, 20, 0));

        // Comparing patch.
        assert!(KernelVersion::new(5, 0, 20) < KernelVersion::new(5, 10, 15));
        assert!(KernelVersion::new(5, 0, 20) > KernelVersion::new(4, 10, 15));

        // Equal.
        assert!(KernelVersion::new(5, 0, 20) == KernelVersion::new(5, 0, 20));
        assert!(KernelVersion::new(5, 0, 20) >= KernelVersion::new(5, 0, 20));
        assert!(KernelVersion::new(5, 0, 20) <= KernelVersion::new(5, 0, 20));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", KernelVersion::new(5, 8, 80)), "5.8.80");
    }
}
