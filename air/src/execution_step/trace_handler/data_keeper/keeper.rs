/*
 * Copyright 2021 Fluence Labs Limited
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use super::ExecutionTrace;
use super::MergeCtx;
use super::TraceSlider;

use air_interpreter_data::InterpreterData;

/// Keeps all necessary data for merging.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct DataKeeper {
    pub(crate) prev_ctx: MergeCtx,
    pub(crate) current_ctx: MergeCtx,
    pub(crate) result_trace: ExecutionTrace,
}

impl DataKeeper {
    pub(crate) fn from_data(prev_data: InterpreterData, current_data: InterpreterData) -> Self {
        let prev_ctx = MergeCtx::from_data(prev_data);
        let current_ctx = MergeCtx::from_data(current_data);

        Self {
            prev_ctx,
            current_ctx,
            result_trace: <_>::default(),
        }
    }

    pub(crate) fn result_states_count(&self) -> usize {
        self.result_trace.len()
    }

    pub(crate) fn prev_slider(&self) -> &TraceSlider {
        &self.prev_ctx.slider
    }

    pub(crate) fn prev_slider_mut(&mut self) -> &mut TraceSlider {
        &mut self.prev_ctx.slider
    }

    pub(crate) fn current_slider(&self) -> &TraceSlider {
        &self.current_ctx.slider
    }

    pub(crate) fn current_slider_mut(&mut self) -> &mut TraceSlider {
        &mut self.current_ctx.slider
    }
}
