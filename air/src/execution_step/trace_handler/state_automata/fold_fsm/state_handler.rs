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
use crate::execution_step::trace_handler::MergeCtxType;
use crate::execution_step::trace_handler::ResolvedFold;

/// This state updater manage to do the same thing as SubTreeStateUpdater in ParFSM,
/// for details please see its detailed comment.
#[derive(Debug, Default, Clone)]
pub(super) struct CtxStateHandler {
    state_pair: CtxStatesPair,
}

impl CtxStateHandler {
    pub(super) fn prepare(
        prev_fold: &ResolvedFold,
        current_fold: &ResolvedFold,
        data_keeper: &DataKeeper,
    ) -> FSMResult<Self> {
        let prev_state = compute_new_state(prev_fold, data_keeper, MergeCtxType::Previous)?;
        let current_state = compute_new_state(current_fold, data_keeper, MergeCtxType::Current)?;
        let state_pair = CtxStatesPair::new(prev_state, current_state);

        let updater = Self { state_pair };
        Ok(updater)
    }

    pub(super) fn set_final_states(self, data_keeper: &mut DataKeeper) {
        update_ctx_states(self.state_pair, data_keeper)
    }
}

fn compute_new_state(fold: &ResolvedFold, data_keeper: &DataKeeper, ctx_type: MergeCtxType) -> FSMResult<CtxState> {
    let ctx = match ctx_type {
        MergeCtxType::Previous => &data_keeper.prev_ctx,
        MergeCtxType::Current => &data_keeper.current_ctx,
    };

    let current_position = ctx.slider.position();
    let current_len = ctx.slider.subtrace_len();

    let pos = current_position
        .checked_add(fold.fold_states_count)
        .ok_or_else(|| StateFSMError::FoldPosOverflow(fold.clone(), current_position, ctx_type))?;

    let subtrace_len = current_len
        .checked_sub(fold.fold_states_count)
        .ok_or_else(|| StateFSMError::FoldLenUnderflow(fold.clone(), current_position, ctx_type))?;

    let total_subtrace_len = ctx.total_subtrace_len() - fold.fold_states_count;

    let state = CtxState::new(pos, subtrace_len, total_subtrace_len);
    Ok(state)
}
