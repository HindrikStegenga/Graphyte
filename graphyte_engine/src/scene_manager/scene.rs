use graphyte_utils::handles::*;
use shard_ecs::*;

pub struct Scene {
    handle: Handle<Scene, u32>,
    registry: Registry,
}

impl Scene {
    pub fn handle(&self) -> Handle<Scene, u32> {
        self.handle
    }
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
    pub fn registry_mut(&mut self) -> &mut Registry {
        &mut self.registry
    }
}

#[derive(Clone)]
pub struct SceneDidBecomeCurrent {
    scene: Handle<Scene, u32>
}

#[derive(Clone)]
pub struct SceneWasCreated {
    scene: Handle<Scene, u32>
}

#[derive(Clone)]
pub struct SceneWasDestroyed {
    scene: Handle<Scene, u32>
}