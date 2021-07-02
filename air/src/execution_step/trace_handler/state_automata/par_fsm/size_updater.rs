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

/// At the end of a Par execution it's needed to reduce subtrace_len of both sliders on a count
/// of seen states. This count could be taken from the Par left and right subtree sizes.
/// This struct manage to save the updated lens and update subtrace_len of sliders.
#[derive(Debug, Default, Clone)]
pub(super) struct SubTraceSizeUpdater {
    pub(self) prev_size: usize,
    pub(self) current_size: usize,
}

impl SubTraceSizeUpdater {
    pub(super) fn from_data_keeper(data_keeper: &DataKeeper, ingredients: MergerParResult) -> FSMResult<Self> {
        let prev_subtree_size = data_keeper.prev_ctx.slider.subtrace_len();
        let prev_size = Self::compute_new_size(prev_subtree_size, ingredients.prev_par)?;

        let current_subtree_size = data_keeper.current_ctx.slider.subtrace_len();
        let current_size = Self::compute_new_size(current_subtree_size, ingredients.current_par)?;

        let updater = Self {
            prev_size,
            current_size,
        };

        Ok(updater)
    }

    pub(super) fn update(self, data_keeper: &mut DataKeeper) -> FSMResult<()> {
        data_keeper.prev_ctx.slider.set_subtrace_len(self.prev_size)?;
        data_keeper.current_ctx.slider.set_subtrace_len(self.current_size)?;

        Ok(())
    }

    fn compute_new_size(initial_size: usize, par_result: Option<ParResult>) -> FSMResult<usize> {
        let par_size = par_result
            .map(|p| p.size().ok_or(StateFSMError::ParLenOverflow(p)))
            .transpose()?
            .unwrap_or_default();

        let new_size = initial_size
            .checked_sub(par_size)
            // unwrap is safe here, because underflow could be caused only if par is Some
            .ok_or_else(|| StateFSMError::ParSubtreeUnderflow(par_result.unwrap(), initial_size))?;

        Ok(new_size)
    }
}
