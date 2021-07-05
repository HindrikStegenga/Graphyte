use std::{error::Error, fmt::Display};

use crate::{archive::AssetArchiveError, vfs::error::VfsError};

#[derive(Debug)]
pub enum AssetSystemError {
    Vfs(VfsError),
    Io(std::io::Error),
    Other(Box<dyn Error>),
    Archive(AssetArchiveError),
}
impl Error for AssetSystemError {}
impl Display for AssetSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetSystemError::Vfs(e) => e.fmt(f),
            AssetSystemError::Io(e) => e.fmt(f),
            AssetSystemError::Other(e) => e.fmt(f),
            AssetSystemError::Archive(e) => e.fmt(f),
        }
    }
}
impl From<std::io::Error> for AssetSystemError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<VfsError> for AssetSystemError {
    fn from(e: VfsError) -> Self {
        Self::Vfs(e)
    }
}
impl From<Box<dyn Error>> for AssetSystemError {
    fn from(e: Box<dyn Error>) -> Self {
        Self::Other(e)
    }
}
impl From<AssetArchiveError> for AssetSystemError {
    fn from(e: AssetArchiveError) -> Self {
        AssetSystemError::Archive(e)
    }
}
