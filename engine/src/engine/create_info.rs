use std::{ffi::CString, num::NonZeroUsize};

use crate::engine_stages::{RenderStageConstructor, UpdateStageConstructor};
use asset_library::asset_system::AssetSystem;
use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConcurrencySettings {
    pub max_async_threads: Option<NonZeroUsize>,
    pub max_worker_thread: Option<NonZeroUsize>,
    pub fallback_worker_threads: NonZeroUsize,
    pub fallback_async_threads: NonZeroUsize,
}

/// Information required to construct an instance of [`Engine`].
pub struct EngineCreateInfo {
    pub asset_system: Option<Box<AssetSystemCreateFn>>,
    pub application_info: ApplicationInfo,
    pub update_tick_rate: u32,
    pub max_skipped_frames: u32,
    pub max_frame_rate: Option<u32>,
    pub concurrency_settings: EngineConcurrencySettings,
    pub update_stages: Vec<Box<UpdateStageConstructor>>,
    pub render_stages: Vec<Box<RenderStageConstructor>>,
}

pub type AssetSystemCreateFn = dyn Fn() -> AssetSystem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationInfo {
    pub application_name: CString,
    pub engine_name: CString,

    pub application_major_version: u32,
    pub application_minor_version: u32,
    pub application_patch_version: u32,

    pub engine_major_version: u32,
    pub engine_minor_version: u32,
    pub engine_patch_version: u32,
}
