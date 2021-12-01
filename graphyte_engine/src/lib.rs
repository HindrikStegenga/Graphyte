pub mod engine;
pub mod engine_stages;
pub mod platform;

pub use engine::{create_info::EngineCreateInfo, result::EngineUpdateResult, Engine};
pub use platform::*;
pub use graphyte_asset_library::asset_system::AssetSystem;

#[cfg(feature = "re_export_utils")]
pub use graphyte_utils::*;

#[cfg(not(feature = "re_export_utils"))]
pub(crate) use magnetar_utils::*;