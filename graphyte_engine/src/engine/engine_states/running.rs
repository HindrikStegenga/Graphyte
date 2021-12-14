use graphyte_asset_library::{dispatch_system::DispatchSystem, resource_system::ResourceSystem};

use super::*;
use crate::{engine::result::*, engine_stages::*, PlatformInterface};
use std::{sync::Arc, time::*};

pub struct Running {
    pub(super) update_stages_runner: UpdateStagesRunner,
    pub(crate) render_stages: Vec<Box<dyn AnyRenderStage>>,
}

impl Into<EngineStateMachine<Suspended>> for EngineStateMachine<Running> {
    fn into(self) -> EngineStateMachine<Suspended> {
        EngineStateMachine {
            shared: self.shared,
            state: Suspended {
                update_stages_runner: self.state.update_stages_runner,
                render_stages: self.state.render_stages,
            },
        }
    }
}

impl EngineStateMachine<Running> {
    pub fn tick(&mut self, interface: &mut dyn PlatformInterface) -> EngineUpdateResult {
        self.shared.internal_resources.timings.frame_start();

        let fixed_update_step_duration = Duration::from_millis(1000)
            / (self.shared.internal_resources.timings.update_tick_rate as u32);

        // Trigger the update thread if necessary.
        let mut n_loops = 0;
        while self.shared.internal_resources.timings.accumulated_time >= fixed_update_step_duration
            && n_loops < (1 + self.shared.internal_resources.timings.max_skipped_frames)
        {
            match self.state.update_stages_runner.update(&mut self.shared) {
                EngineUpdateResult::Ok => {}
                result => {
                    return result;
                }
            }

            self.shared.internal_resources.timings.accumulated_time -= fixed_update_step_duration;
            n_loops += 1;
            self.shared.internal_resources.timings.update_counter += 1;
            self.shared
                .internal_resources
                .timings
                .last_fixed_update_instant =
                self.shared.internal_resources.timings.frame_start_instant;
        }


        // Trigger the render thread.
        for stage in &mut self.state.render_stages {
            match stage.render(RenderStageUpdateInput::new(interface)) {
                EngineUpdateResult::Ok => {}
                result => {
                    return result;
                }
            }
        }

        self.shared.internal_resources.timings.frame_end();

        EngineUpdateResult::Ok
    }
}
