use std::{fs::read_dir, path::Path};
mod error;
use serde::de::DeserializeOwned;

use crate::{
    archive::*,
    vfs::*,
    vfs::{archive_mount_point::ArchiveMountPoint, physical_mount_point::VfsPhysicalMountPoint},
    *,
};

use self::error::AssetSystemError;

pub struct AssetSystem {
    vfs: VirtualFileSystem,
}

impl Default for AssetSystem {
    fn default() -> Self {
        Self {
            vfs: Default::default(),
        }
    }
}

impl AssetSystem {
    /// Deserializes an asset into the provided type, allocates internal byte buffer temprorarily.
    /// The asset system only supports YAML, CBOR, TOML and JSON formats.
    // Might required specific features to be enabled.
    pub fn load_asset_as_type<T: DeserializeOwned>(
        &self,
        mount_point: impl AsRef<str>,
        identifier: impl AsRef<str>,
    ) -> Result<T, AssetSystemError> {
        let mut buffer = Vec::new();
        self.load_asset_as_type_using_buffer(mount_point, identifier, &mut buffer)
    }

    /// Deserializes an asses into the provided type, using the provided buffer as intermediate.
    pub fn load_asset_as_type_using_buffer<T: DeserializeOwned>(
        &self,
        mount_point: impl AsRef<str>,
        identifier: impl AsRef<str>,
        buffer: &mut Vec<u8>,
    ) -> Result<T, AssetSystemError> {
        let descriptor = self.load_asset_as_blob_into(&mount_point, &identifier, buffer)?;

        match descriptor.format() {
            "yaml" | "yml" => {
                serde_yaml::from_slice(&buffer).map_err(|e| AssetSystemError::Other(Box::from(e)))
            }
            "cbor" => {
                serde_cbor::from_slice(&buffer).map_err(|e| AssetSystemError::Other(Box::from(e)))
            }
            #[cfg(feature = "format_json")]
            "json" => {
                serde_json::from_slice(&buffer).map_err(|e| AssetSystemError::Other(Box::from(e)))
            }
            #[cfg(feature = "format_toml")]
            "toml" => toml::from_slice(&buffer).map_err(|e| AssetSystemError::Other(Box::from(e))),
            _ => {
                tagged_warn!(
                    "Asset System",
                    "Tried to load asset {} with unknown format {} from {}.",
                    identifier.as_ref(),
                    descriptor.format(),
                    mount_point.as_ref()
                );
                Err(AssetSystemError::UnknownAssetFormat)
            }
        }
    }

    pub fn load_asset_as_blob_into(
        &self,
        mount_point: impl AsRef<str>,
        identifier: impl AsRef<str>,
        buffer: &mut Vec<u8>,
    ) -> Result<AssetDescriptor, AssetSystemError> {
        self.vfs
            .read_file_into(mount_point, identifier, buffer)
            .map_err(|e| e.into())
    }

    pub fn load_files_from_directory(
        &mut self,
        directory: impl AsRef<Path>,
        mount_point: impl AsRef<str>,
    ) -> Result<(), AssetSystemError> {
        let mnt = VfsPhysicalMountPoint::new(&mount_point, &directory)?;
        if !self.vfs.mount(mnt) {
            tagged_warn!(
                "Asset System",
                "Directory mount point was not mounted: {:#?} mount point: {:#?}",
                directory.as_ref(),
                mount_point.as_ref()
            );
        }
        tagged_success!(
            "Asset System",
            "Mounted files from directory: {:#?} mount point: {:#?}",
            directory.as_ref(),
            mount_point.as_ref()
        );
        Ok(())
    }

    pub fn load_archives_from_directory(
        &mut self,
        directory: impl AsRef<Path>,
        file_extension: impl AsRef<str>,
    ) -> Result<(), AssetSystemError> {
        let dir = read_dir(directory.as_ref())?;
        let valid_dir_entries = dir
            .filter_map(|d| match d {
                Err(e) => {
                    tagged_warn!("Asset System", "Invalid directory entry found: {}", e);
                    None
                }
                Ok(v) => Some(v),
            })
            .collect::<Vec<_>>();
        let valid_dir_entries = valid_dir_entries
            .iter()
            .filter_map(|d| match d.metadata() {
                Ok(m) => match m.is_file() {
                    true => match d.path().extension() {
                        Some(extension) => match extension.to_str() {
                            Some(s) => {
                                if s == file_extension.as_ref() {
                                    Some(d)
                                } else {
                                    None
                                }
                            }
                            None => None,
                        },
                        None => None,
                    },
                    false => None,
                },
                Err(e) => {
                    tagged_warn!("Asset System", "Could not retrieve file metadata: {}", e);
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut counter = 0;
        for dir_entry in valid_dir_entries {
            let archive = AssetArchive::read_from_file(dir_entry.path())?;
            for mount_point in archive.header().mount_points() {
                let physical_mount =
                    ArchiveMountPoint::new(archive.path().into(), mount_point.clone());
                if !self.vfs.mount(physical_mount) {
                    tagged_warn!(
                        "Asset System",
                        "Archive mount point was not mounted: {:#?}",
                        archive.path()
                    );
                }
            }

            counter += 1;
        }
        tagged_success!(
            "Asset System",
            "Loaded {} asset archives from: {:#?}",
            counter,
            directory.as_ref()
        );

        Ok(())
    }
}