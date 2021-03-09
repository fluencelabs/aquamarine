/*
 * Copyright 2020 Fluence Labs Limited
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

use super::MergeCtx;
use super::MergeResult;
use crate::preparation::CallResult;
use crate::preparation::DataMergingError;
use crate::preparation::ExecutedState;
use crate::preparation::ExecutionTrace;
use crate::preparation::FoldResult;
use crate::preparation::ParResult;
use crate::JValue;

use air_parser::ast::Instruction;

use std::cell::RefCell;
use std::rc::Rc;

type MergingStream = Rc<RefCell<Vec<Rc<JValue>>>>;

#[derive(Debug, PartialEq)]
pub(crate) struct TraceMerger<'i> {
    prev_ctx: MergeCtx,
    current_ctx: MergeCtx,
    result_trace: ExecutionTrace,
    aqua: &'i Instruction<'i>,
}

impl<'i> TraceMerger<'i> {
    pub(crate) fn new(prev_trace: ExecutionTrace, current_trace: ExecutionTrace, aqua: &'i Instruction<'i>) -> Self {
        let max_trace_len = std::cmp::max(prev_trace.len(), current_trace.len());
        let result_trace = ExecutionTrace::with_capacity(max_trace_len);

        let prev_ctx = MergeCtx::new(prev_trace);
        let current_ctx = MergeCtx::new(current_trace);

        Self {
            prev_ctx,
            current_ctx,
            result_trace,
            aqua,
        }
    }

    pub(crate) fn merge(mut self) -> MergeResult<ExecutionTrace> {
        use crate::log_targets::EXECUTED_TRACE_MERGE;

        self.merge_subtree()?;

        log::trace!(target: EXECUTED_TRACE_MERGE, "merged trace: {:?}", self.result_trace);

        Ok(self.result_trace)
    }

    fn merge_subtree(&mut self) -> MergeResult<()> {
        use DataMergingError::IncompatibleExecutedStates;
        use ExecutedState::*;

        loop {
            let prev_state = self.prev_ctx.slider.next_state();
            let current_state = self.current_ctx.slider.next_state();

            match (prev_state, current_state) {
                (Some(Call(prev_call)), Some(Call(current_call))) => self.merge_calls(prev_call, current_call)?,
                (Some(Par(prev_par)), Some(Par(current_par))) => self.merge_pars(prev_par, current_par)?,
                (Some(Fold(prev_fold)), Some(Fold(current_fold))) => self.merge_folds(prev_fold, current_fold)?,
                (None, Some(s)) => {
                    self.result_trace.push_back(s);

                    let current_states = self.current_ctx.slider.drain_interval()?;
                    self.result_trace.extend(current_states);
                    break;
                }
                (Some(s), None) => {
                    self.result_trace.push_back(s);

                    let prev_states = self.prev_ctx.slider.drain_interval()?;
                    self.result_trace.extend(prev_states);
                    break;
                }
                (None, None) => break,
                // this match arm represents (Call, Par), (Par, Call), (Fold, Call) ... states
                (Some(prev_state), Some(current_state)) => {
                    return Err(IncompatibleExecutedStates(prev_state, current_state))
                }
            }
        }

        Ok(())
    }

    fn merge_calls(&mut self, prev_call_result: CallResult, current_call_result: CallResult) -> MergeResult<()> {
        use CallResult::*;
        use DataMergingError::IncompatibleCallResults;

        let call_result = match (&prev_call_result, &current_call_result) {
            (CallServiceFailed(prev_ret_code, prev_err_msg), CallServiceFailed(ret_code, err_msg)) => {
                if prev_ret_code != ret_code || prev_err_msg != err_msg {
                    return Err(IncompatibleCallResults(prev_call_result, current_call_result));
                }
                current_call_result
            }
            (RequestSentBy(_), CallServiceFailed(..)) => current_call_result,
            (CallServiceFailed(..), RequestSentBy(_)) => prev_call_result,
            (RequestSentBy(prev_sender), RequestSentBy(sender)) => {
                if prev_sender != sender {
                    return Err(IncompatibleCallResults(prev_call_result, current_call_result));
                }

                prev_call_result
            }
            (RequestSentBy(_), Executed(..)) => {
                self.current_ctx.maybe_update_stream(&current_call_result);
                current_call_result
            }
            (Executed(..), RequestSentBy(_)) => {
                self.prev_ctx.maybe_update_stream(&prev_call_result);
                prev_call_result
            }
            (Executed(prev_result, prev_type), Executed(current_result, current_type)) => {
                if prev_result != current_result || prev_type != current_type {
                    return Err(IncompatibleCallResults(prev_call_result, current_call_result));
                }

                self.prev_ctx.maybe_update_stream(&prev_call_result);
                self.current_ctx.maybe_update_stream(&current_call_result);

                prev_call_result
            }
            (CallServiceFailed(..), Executed(..)) => {
                return Err(IncompatibleCallResults(prev_call_result, current_call_result))
            }
            (Executed(..), CallServiceFailed(..)) => {
                return Err(IncompatibleCallResults(prev_call_result, current_call_result))
            }
        };

        self.result_trace.push_back(ExecutedState::Call(call_result));

        Ok(())
    }

    fn merge_pars(&mut self, prev_par: ParResult, current_par: ParResult) -> MergeResult<()> {
        let prev_subtree_size = self.prev_ctx.slider.subtree_size();
        let current_subtree_size = self.current_ctx.slider.subtree_size();

        let par_position = self.result_trace.len();
        // place a temporary Par value to avoid insertion in the middle
        self.result_trace.push_back(ExecutedState::Par(ParResult(0, 0)));

        let len_before_merge = self.result_trace.len();

        self.prev_ctx.slider.set_interval_len(prev_par.0);
        self.current_ctx.slider.set_interval_len(current_par.0);
        self.merge_subtree()?;

        let left_par_size = self.result_trace.len() - len_before_merge;

        self.prev_ctx.slider.set_interval_len(prev_par.1);
        self.current_ctx.slider.set_interval_len(current_par.1);
        self.merge_subtree()?;

        let right_par_size = self.result_trace.len() - left_par_size - len_before_merge;

        // update the temporary Par with final values
        self.result_trace[par_position] = ExecutedState::Par(ParResult(left_par_size, right_par_size));

        self.prev_ctx
            .slider
            .set_interval_len(prev_subtree_size - prev_par.0 - prev_par.1);
        self.current_ctx
            .slider
            .set_interval_len(current_subtree_size - current_par.0 - current_par.1);

        Ok(())
    }

    fn merge_folds(&mut self, prev_fold: FoldResult, current_fold: FoldResult) -> MergeResult<()> {
        use std::collections::HashSet;

        let _prev_subtree_size = self.prev_ctx.slider.subtree_size();
        let _current_subtree_size = self.current_ctx.slider.subtree_size();

        let (prev_stream, current_stream) = self.extract_streams(prev_fold.0, current_fold.0)?;

        let mut current_used_values = HashSet::new();
        for (prev_pos, value) in prev_stream.borrow().iter().enumerate() {
            let mut current_pos: Option<usize> = None;
            for stream_id in 0..current_stream.borrow().len() {
                if &current_stream.borrow()[stream_id] == value && !current_used_values.contains(&stream_id) {
                    current_used_values.insert(stream_id);
                    current_pos = Some(stream_id);
                }
            }

            let prev_fold_begin = prev_fold.1[prev_pos].left_begin;
            let prev_fold_end = prev_fold.1[prev_pos].left_end;
            self.prev_ctx.slider.adjust_position(prev_fold_begin);
            self.prev_ctx.slider.set_interval_len(prev_fold_end - prev_fold_begin);

            if let Some(pos) = current_pos {
                let current_fold_begin = current_fold.1[pos].left_begin;
                let current_fold_end = current_fold.1[pos].left_end;
                self.current_ctx.slider.adjust_position(current_fold_begin);
                self.current_ctx
                    .slider
                    .set_interval_len(current_fold_end - current_fold_begin);
            } else {
                self.current_ctx.slider.set_interval_len(0);
            }

            self.merge_subtree()?;
        }

        // merge values that has only the second stream
        for stream_id in 0..current_fold.1.len() {
            if current_used_values.contains(&stream_id) {
                continue;
            }

            let current_fold_begin = current_fold.1[stream_id].left_begin;
            let current_fold_end = current_fold.1[stream_id].left_end;
            self.current_ctx.slider.adjust_position(current_fold_begin);
            self.current_ctx
                .slider
                .set_interval_len(current_fold_end - current_fold_begin);

            self.prev_ctx.slider.set_interval_len(0);

            self.merge_subtree()?;
        }

        let mut current_used_values = HashSet::new();
        for (prev_pos, value) in prev_stream.borrow().iter().rev().enumerate() {
            let mut current_pos: Option<usize> = None;
            for stream_id in 0..current_stream.borrow().len() {
                if &current_stream.borrow()[stream_id] == value && !current_used_values.contains(&stream_id) {
                    current_used_values.insert(stream_id);
                    current_pos = Some(stream_id);
                }
            }

            let prev_fold_begin = prev_fold.1[prev_pos].right_begin;
            let prev_fold_end = prev_fold.1[prev_pos].right_end;
            self.prev_ctx.slider.adjust_position(prev_fold_begin);
            self.prev_ctx.slider.set_interval_len(prev_fold_end - prev_fold_begin);

            if let Some(pos) = current_pos {
                let current_fold_begin = current_fold.1[pos].right_begin;
                let current_fold_end = current_fold.1[pos].right_end;
                self.current_ctx.slider.adjust_position(current_fold_begin);
                self.current_ctx
                    .slider
                    .set_interval_len(current_fold_end - current_fold_begin);
            } else {
                self.current_ctx.slider.set_interval_len(0);
            }

            self.merge_subtree()?;
        }

        // merge values that has only the second stream
        for stream_id in current_fold.1.len()..0 {
            if current_used_values.contains(&stream_id) {
                continue;
            }

            let current_fold_begin = current_fold.1[stream_id].right_begin;
            let current_fold_end = current_fold.1[stream_id].right_end;
            self.current_ctx.slider.adjust_position(current_fold_begin);
            self.current_ctx
                .slider
                .set_interval_len(current_fold_end - current_fold_begin);

            self.prev_ctx.slider.set_interval_len(0);

            self.merge_subtree()?;
        }

        Ok(())
    }

    fn extract_streams(
        &self,
        prev_stream_name: String,
        current_stream_name: String,
    ) -> MergeResult<(MergingStream, MergingStream)> {
        if prev_stream_name != current_stream_name {
            return Err(DataMergingError::IncompatibleFoldIterableNames(
                prev_stream_name,
                current_stream_name,
            ));
        }

        let prev_stream = extract_stream(&self.prev_ctx, &prev_stream_name);
        let current_stream = extract_stream(&self.current_ctx, &current_stream_name);

        Ok((prev_stream, current_stream))
    }
}

fn extract_stream(merge_ctx: &MergeCtx, stream_name: &str) -> MergingStream {
    match merge_ctx.streams.get(stream_name) {
        Some(stream) => stream.clone(),
        // this one means iterable with no values
        None => Rc::new(RefCell::new(vec![])),
    }
}