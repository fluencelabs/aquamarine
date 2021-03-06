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

use super::CallResult;
use super::DataMergingError;
use super::ExecutedState;
use super::ExecutionTrace;
use super::ParResult;
use crate::log_targets::EXECUTED_TRACE_MERGE;

type MergeResult<T> = Result<T, DataMergingError>;

pub(super) fn merge_execution_traces(
    mut prev_trace: ExecutionTrace,
    mut current_trace: ExecutionTrace,
) -> MergeResult<ExecutionTrace> {
    let mut merged_trace = ExecutionTrace::new();

    let prev_subtree_size = prev_trace.len();
    let current_subtree_size = current_trace.len();

    merge_subtree(
        &mut prev_trace,
        prev_subtree_size,
        &mut current_trace,
        current_subtree_size,
        &mut merged_trace,
    )?;

    log::trace!(target: EXECUTED_TRACE_MERGE, "merged trace: {:?}", merged_trace);

    Ok(merged_trace)
}

fn merge_subtree(
    prev_trace: &mut ExecutionTrace,
    mut prev_subtree_size: usize,
    current_trace: &mut ExecutionTrace,
    mut current_subtree_size: usize,
    result_trace: &mut ExecutionTrace,
) -> MergeResult<()> {
    use DataMergingError::ExecutedTraceTooSmall;
    use DataMergingError::IncompatibleExecutedStates;
    use ExecutedState::*;

    loop {
        let prev_state = if prev_subtree_size != 0 {
            prev_subtree_size -= 1;
            prev_trace.pop_front()
        } else {
            None
        };

        let current_state = if current_subtree_size != 0 {
            current_subtree_size -= 1;
            current_trace.pop_front()
        } else {
            None
        };

        match (prev_state, current_state) {
            (Some(Call(prev_call)), Some(Call(call))) => {
                let resulted_call = merge_call(prev_call, call)?;
                result_trace.push_back(Call(resulted_call));
            }
            (Some(Par(ParResult(prev_left, prev_right))), Some(Par(ParResult(current_left, current_right)))) => {
                let par_position = result_trace.len();
                // place temporary Par value to avoid insert in the middle
                result_trace.push_back(ExecutedState::par(0, 0));

                let before_result_len = result_trace.len();

                merge_subtree(prev_trace, prev_left, current_trace, current_left, result_trace)?;
                let left_par_size = result_trace.len() - before_result_len;

                merge_subtree(prev_trace, prev_right, current_trace, current_right, result_trace)?;
                let right_par_size = result_trace.len() - left_par_size - before_result_len;

                // update temporary Par with final values
                result_trace[par_position] = ExecutedState::par(left_par_size, right_par_size);

                prev_subtree_size -= prev_left + prev_right;
                current_subtree_size -= current_left + current_right;
            }
            (None, Some(s)) => {
                if current_trace.len() < current_subtree_size {
                    return Err(ExecutedTraceTooSmall(current_trace.len(), current_subtree_size));
                }

                result_trace.push_back(s);
                result_trace.extend(current_trace.drain(..current_subtree_size));
                break;
            }
            (Some(s), None) => {
                if prev_trace.len() < prev_subtree_size {
                    return Err(ExecutedTraceTooSmall(prev_trace.len(), prev_subtree_size));
                }

                result_trace.push_back(s);
                result_trace.extend(prev_trace.drain(..prev_subtree_size));
                break;
            }
            (None, None) => break,
            // this match arn represents (Call, Par) and (Par, Call) states
            (Some(prev_state), Some(current_state)) => {
                return Err(IncompatibleExecutedStates(prev_state, current_state))
            }
        }
    }

    Ok(())
}

fn merge_call(prev_call_result: CallResult, current_call_result: CallResult) -> MergeResult<CallResult> {
    use super::CallResult::*;
    use super::DataMergingError::IncompatibleCallResults;

    match (&prev_call_result, &current_call_result) {
        (CallServiceFailed(prev_ret_code, prev_err_msg), CallServiceFailed(ret_code, err_msg)) => {
            if prev_ret_code != ret_code || prev_err_msg != err_msg {
                return Err(IncompatibleCallResults(prev_call_result, current_call_result));
            }
            Ok(current_call_result)
        }
        (RequestSentBy(_), CallServiceFailed(..)) => Ok(current_call_result),
        (CallServiceFailed(..), RequestSentBy(_)) => Ok(prev_call_result),
        (RequestSentBy(prev_sender), RequestSentBy(sender)) => {
            if prev_sender != sender {
                return Err(IncompatibleCallResults(prev_call_result, current_call_result));
            }

            Ok(prev_call_result)
        }
        (RequestSentBy(_), Executed(..)) => Ok(current_call_result),
        (Executed(..), RequestSentBy(_)) => Ok(prev_call_result),
        (Executed(prev_result), Executed(result)) => {
            if prev_result != result {
                return Err(IncompatibleCallResults(prev_call_result, current_call_result));
            }

            Ok(prev_call_result)
        }
        (CallServiceFailed(..), Executed(..)) => Err(IncompatibleCallResults(prev_call_result, current_call_result)),
        (Executed(..), CallServiceFailed(..)) => Err(IncompatibleCallResults(prev_call_result, current_call_result)),
    }
}
