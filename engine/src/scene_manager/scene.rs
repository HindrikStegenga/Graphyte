use crate::scene_manager::SceneHandle;
use shard_ecs::*;
use utils::handles::*;

pub struct Scene {
    handle: SceneHandle,
    registry: Registry,
}

impl Scene {
    pub(super) fn new(handle: SceneHandle) -> Scene {
        Self {
            handle,
            registry: Default::default(),
        }
    }

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

#[allow(unused)]
#[derive(Clone)]
pub struct SceneDidBecomeCurrent {
    pub scene: SceneHandle,
}

#[allow(unused)]
#[derive(Clone)]
pub struct SceneWasCreated {
    pub scene: SceneHandle,
}

#[allow(unused)]
#[derive(Clone)]
pub struct SceneWasDestroyed {
    pub scene: SceneHandle,
}
