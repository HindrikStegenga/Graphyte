pub mod archive_mount_point;
pub mod error;
pub mod physical_mount_point;

use crate::AssetDescriptor;
use error::VfsError;
use std::collections::HashMap;
use utils::*;

pub trait VfsMountPoint: Send + 'static {
    fn identifier(&self) -> &str;
    fn has_file(&self, identifier: &str) -> bool;
    fn get_asset_descriptor(&self, identifier: &str) -> Option<AssetDescriptor>;
    fn load_asset_into(
        &self,
        identifier: &str,
        buffer: &mut Vec<u8>,
    ) -> Result<AssetDescriptor, VfsError>;
    fn version(&self) -> u64;
}

pub struct VirtualFileSystem {
    mounts: HashMap<String, Vec<Box<dyn VfsMountPoint + Send>>>,
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        Self {
            mounts: Default::default(),
        }
    }
}

impl VirtualFileSystem {
    /// Mounts a new virtual mountpoint into the virtual file system.
    pub fn mount(&mut self, mountpoint: impl VfsMountPoint) -> bool {
        t_info!(
            "Mounting mountpoint: {} version: {}",
            mountpoint.identifier(),
            mountpoint.version()
        );
        match self.mounts.get_mut(mountpoint.identifier()) {
            Some(v) => match v.binary_search_by_key(&mountpoint.version(), |e| e.version()) {
                Ok(_) => {
                    t_warn!("Attempted to mount a mountpoint with identical version to the same mountpoint.");
                    return false;
                }
                Err(insertion_idx) => {
                    v.insert(insertion_idx, Box::from(mountpoint));
                }
            },
            None => {
                let mut v: Vec<Box<dyn VfsMountPoint + Send>> = Vec::with_capacity(4);
                let key = mountpoint.identifier().into();
                v.push(Box::new(mountpoint));
                self.mounts.insert(key, v);
            }
        }
        t_info!("Mounting successfull!");
        true
    }

    pub fn read_file(
        &self,
        mount_point: impl AsRef<str>,
        file_identifier: impl AsRef<str>,
    ) -> Result<(Vec<u8>, AssetDescriptor), VfsError> {
        let mut buffer = Vec::new();
        self.read_file_into(mount_point, file_identifier, &mut buffer)
            .map_err(|e| e.into())
            .map(|a| (buffer, a))
    }

    pub fn read_file_into(
        &self,
        mount_point: impl AsRef<str>,
        file_identifier: impl AsRef<str>,
        mut buffer: &mut Vec<u8>,
    ) -> Result<AssetDescriptor, VfsError> {
        let mounts = match self.mounts.get(&mount_point.as_ref().to_lowercase()) {
            Some(v) => v,
            None => return Err(VfsError::MountpointNotFound),
        };
        for mount in mounts.iter().rev() {
            match mount.load_asset_into(&file_identifier.as_ref().to_lowercase(), &mut buffer) {
                Ok(a) => return Ok(a),
                Err(VfsError::FileNotFound) => continue,
                Err(e) => {
                    t_warn!("Error occurred while loading file: {}", e);
                    return Err(e);
                }
            }
        }
        Err(VfsError::FileNotFound)
    }
}
