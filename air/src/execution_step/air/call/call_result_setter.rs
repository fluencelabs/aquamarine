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
use crate::exec_err;
use crate::execution_step::execution_context::*;
use crate::execution_step::trace_handler::TraceHandler;
use crate::execution_step::Generation;
use crate::execution_step::Stream;
use crate::execution_step::Variable;

use air_interpreter_data::CallResult;
use air_interpreter_data::SCALAR_GENERATION;
use air_parser::ast::CallOutputValue;

use std::cell::RefCell;
use std::collections::hash_map::Entry::{Occupied, Vacant};

// TODO: refactor this function, it receives and produces generation id.
/// Writes result of a local `Call` instruction to `ExecutionCtx` at `output`.
pub(crate) fn set_local_call_result<'i>(
    executed_result: ResolvedCallResult,
    generation: Generation,
    output: &CallOutputValue<'i>,
    exec_ctx: &mut ExecutionCtx<'i>,
) -> ExecutionResult<u32> {
    let generation = match output {
        CallOutputValue::Variable(Variable::Scalar(name)) => {
            set_scalar_result(executed_result, name, exec_ctx)?;
            SCALAR_GENERATION
        }
        CallOutputValue::Variable(Variable::Stream(name)) => {
            set_stream_result(executed_result, generation, name.to_string(), exec_ctx)?
        }
        CallOutputValue::None => SCALAR_GENERATION,
    };

    Ok(generation)
}

macro_rules! shadowing_allowed(
    ($exec_ctx:ident, $entry:ident) => { {
        // check that current execution_step flow is inside a fold block
        if $exec_ctx.met_folds.is_empty() {
            // shadowing is allowed only inside fold blocks
            return exec_err!(ExecutionError::MultipleVariablesFound($entry.key().clone()));
        }

        match $entry.get() {
            AValue::JValueRef(_) => {}
            // shadowing is allowed only for scalar values
            _ => return exec_err!(ExecutionError::NonScalarShadowing($entry.key().clone())),
        };

        ExecutionResult::Ok(())
    }}
);

fn set_scalar_result<'i>(
    executed_result: ResolvedCallResult,
    scalar_name: &'i str,
    exec_ctx: &mut ExecutionCtx<'i>,
) -> ExecutionResult<()> {
    meet_scalar(scalar_name, executed_result.clone(), exec_ctx)?;

    match exec_ctx.data_cache.entry(scalar_name.to_string()) {
        Vacant(entry) => {
            entry.insert(AValue::JValueRef(executed_result));
        }
        Occupied(mut entry) => {
            // the macro instead of a function because of borrowing
            shadowing_allowed!(exec_ctx, entry)?;
            entry.insert(AValue::JValueRef(executed_result));
        }
    };

    Ok(())
}

/// Inserts meet variable name into met calls in fold to allow shadowing.
fn meet_scalar<'i>(
    scalar_name: &'i str,
    executed_result: ResolvedCallResult,
    exec_ctx: &mut ExecutionCtx<'i>,
) -> ExecutionResult<()> {
    if let Some(fold_block_name) = exec_ctx.met_folds.back() {
        let fold_state = match exec_ctx.data_cache.get_mut(*fold_block_name) {
            Some(AValue::JValueFoldCursor(fold_state)) => fold_state,
            _ => unreachable!("fold block data must be represented as fold cursor"),
        };

        fold_state.met_variables.insert(scalar_name, executed_result);
    }

    Ok(())
}

fn set_stream_result(
    executed_result: ResolvedCallResult,
    generation: Generation,
    stream_name: String,
    exec_ctx: &mut ExecutionCtx<'_>,
) -> ExecutionResult<u32> {
    use ExecutionError::IncompatibleAValueType;

    let generation = match exec_ctx.data_cache.entry(stream_name) {
        Occupied(mut entry) => match entry.get_mut() {
            // if result is an array, insert result to the end of the array
            AValue::StreamRef(stream) => stream.borrow_mut().add_value(executed_result, generation)?,
            v => return exec_err!(IncompatibleAValueType(format!("{}", v), String::from("Array"))),
        },
        Vacant(entry) => {
            let stream = Stream::from_value(executed_result);
            entry.insert(AValue::StreamRef(RefCell::new(stream)));
            0
        }
    };

    Ok(generation)
}

/// Writes an executed state of a particle being sent to remote node.
pub(crate) fn set_remote_call_result<'i>(
    peer_pk: String,
    exec_ctx: &mut ExecutionCtx<'i>,
    trace_ctx: &mut TraceHandler,
) {
    exec_ctx.next_peer_pks.push(peer_pk);
    exec_ctx.subtree_complete = false;

    let new_call_result = CallResult::RequestSentBy(exec_ctx.current_peer_id.clone());
    trace_ctx.meet_call_end(new_call_result);
}
