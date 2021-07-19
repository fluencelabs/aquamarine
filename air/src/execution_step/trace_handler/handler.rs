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

use super::*;
use crate::log_targets::EXECUTED_STATE_CHANGING;
use merger::*;

use air_interpreter_data::InterpreterData;
use air_parser::ast::CallOutputValue;

#[derive(Debug, Default)]
pub(crate) struct TraceHandler {
    pub(crate) data_keeper: DataKeeper,
    fsm_keeper: FSMKeeper,
}

impl TraceHandler {
    pub(crate) fn from_data(prev_data: InterpreterData, current_data: InterpreterData) -> Self {
        let data_keeper = DataKeeper::from_data(prev_data, current_data);

        Self {
            data_keeper,
            fsm_keeper: <_>::default(),
        }
    }

    /// Returns size of elements inside result trace and intended to provide
    /// a position of next inserted elements.
    pub(crate) fn trace_pos(&self) -> usize {
        self.data_keeper.result_trace.len()
    }

    pub(crate) fn into_result_trace(self) -> ExecutionTrace {
        self.data_keeper.result_trace
    }

    pub(crate) fn as_result_trace(&self) -> &ExecutionTrace {
        &self.data_keeper.result_trace
    }

    pub(crate) fn subtree_sizes(&self) -> (usize, usize) {
        let prev_len = self.data_keeper.prev_slider().subtrace_len();
        let current_len = self.data_keeper.current_slider().subtrace_len();

        (prev_len, current_len)
    }
}

impl TraceHandler {
    /// Should be called at the beginning of a call execution.
    pub(crate) fn meet_call_start(
        &mut self,
        output_value: &CallOutputValue<'_>,
    ) -> TraceHandlerResult<MergerCallResult> {
        try_merge_next_state_as_call(&mut self.data_keeper, output_value).map_err(Into::into)
    }

    /// Should be called when a call instruction was executed successfully. It adds the supplied
    /// state to the result trace.
    pub(crate) fn meet_call_end(&mut self, call_result: CallResult) {
        log::trace!(
            target: EXECUTED_STATE_CHANGING,
            "  adding new call executed state {:?}",
            call_result
        );
        self.data_keeper.result_trace.push(ExecutedState::Call(call_result));
    }
}

impl TraceHandler {
    pub(crate) fn meet_par_start(&mut self) -> TraceHandlerResult<()> {
        let ingredients = merger::try_merge_next_state_as_par(&mut self.data_keeper)?;
        let par_fsm = ParFSM::from_left_started(ingredients, &mut self.data_keeper)?;
        self.fsm_keeper.push_par(par_fsm);

        Ok(())
    }

    pub(crate) fn meet_par_subtree_end(&mut self, subtree_type: SubtreeType) -> TraceHandlerResult<()> {
        match subtree_type {
            SubtreeType::Left => {
                let par_fsm = self.fsm_keeper.last_par()?;
                par_fsm.left_completed(&mut self.data_keeper)?;
            }
            SubtreeType::Right => {
                let par_fsm = self.fsm_keeper.pop_par()?;
                par_fsm.right_completed(&mut self.data_keeper)?;
            }
        }

        Ok(())
    }

    pub(crate) fn meet_par_subtree_end_with_error(&mut self, subtree_type: SubtreeType) -> TraceHandlerResult<()> {
        match subtree_type {
            SubtreeType::Left => {
                let par_fsm = self.fsm_keeper.last_par()?;
                par_fsm.left_completed_with_error(&mut self.data_keeper);
            }
            SubtreeType::Right => {
                let par_fsm = self.fsm_keeper.pop_par()?;
                par_fsm.right_completed_with_error(&mut self.data_keeper);
            }
        }

        Ok(())
    }
}

impl TraceHandler {
    pub(crate) fn meet_fold_start(&mut self, fold_id: String) -> TraceHandlerResult<()> {
        let ingredients = try_merge_next_state_as_fold(&mut self.data_keeper)?;
        let fold_fsm = FoldFSM::from_fold_start(ingredients, &mut self.data_keeper)?;
        self.fsm_keeper.add_fold(fold_id, fold_fsm);

        Ok(())
    }

    pub(crate) fn meet_iteration_start(&mut self, fold_id: &str, value: &ValueAndPos) -> TraceHandlerResult<()> {
        let fold_fsm = self.fsm_keeper.fold_mut(fold_id)?;
        fold_fsm.meet_iteration_start(value, &mut self.data_keeper)?;

        Ok(())
    }

    pub(crate) fn meet_iteration_end(&mut self, fold_id: &str) -> TraceHandlerResult<()> {
        let fold_fsm = self.fsm_keeper.fold_mut(fold_id)?;
        fold_fsm.meet_iteration_end(&mut self.data_keeper);

        Ok(())
    }

    pub(crate) fn meet_back_iterator(&mut self, fold_id: &str) -> TraceHandlerResult<()> {
        let fold_fsm = self.fsm_keeper.fold_mut(fold_id)?;
        fold_fsm.meet_back_iterator(&mut self.data_keeper)?;

        Ok(())
    }

    pub(crate) fn meet_generation_end(&mut self, fold_id: &str) -> TraceHandlerResult<()> {
        let fold_fsm = self.fsm_keeper.fold_mut(fold_id)?;
        fold_fsm.meet_generation_end(&mut self.data_keeper);

        Ok(())
    }

    pub(crate) fn meet_fold_end(&mut self, fold_id: &str) -> TraceHandlerResult<()> {
        let fold_fsm = self.fsm_keeper.extract_fold(fold_id)?;
        fold_fsm.meet_fold_end(&mut self.data_keeper);

        Ok(())
    }

    pub(crate) fn fold_end_with_error(&mut self, fold_id: &str) {
        // unwrap here is used because this function must be called from a fold block with
        // corresponding fold_id and since it's a error handling, it's better not to produce
        // a new error
        let fold_fsm = self.fsm_keeper.extract_fold(fold_id).unwrap();
        fold_fsm.fold_end_with_error(&mut self.data_keeper);
    }
}
