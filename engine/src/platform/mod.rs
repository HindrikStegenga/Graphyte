use crate::engine::controller::EngineController;

mod interface;
mod messages;
mod window;

pub use interface::*;
pub use messages::*;
pub use window::*;

/// Trait that is used to control the state of the game engine and interact with the OS windowing library.
pub trait Platform {
    /// Execute this function to run the game engine on this platform.
    fn run(self, controller: EngineController);
}
