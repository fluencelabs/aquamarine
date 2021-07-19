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

#![allow(unused_unsafe)] // for wasm_bindgen target where calling FFI is safe

use super::call_result_setter::*;
use super::triplet::Triplet;
use super::utils::*;
use super::*;
use crate::build_targets::CallServiceResult;
use crate::build_targets::CALL_SERVICE_SUCCESS;
use crate::execution_step::trace_handler::MergerCallResult;
use crate::execution_step::trace_handler::TraceHandler;
use crate::execution_step::Generation;
use crate::JValue;
use crate::ResolvedTriplet;
use crate::SecurityTetraplet;

use air_interpreter_data::CallResult;
use air_parser::ast::{CallInstrArgValue, CallOutputValue};

use crate::execution_step::air::ResolvedCallResult;
use std::rc::Rc;

/// Represents Call instruction with resolved internal parts.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct ResolvedCall<'i> {
    triplet: Rc<ResolvedTriplet>,
    function_arg_paths: Rc<Vec<CallInstrArgValue<'i>>>,
    output: CallOutputValue<'i>,
}

#[derive(Debug, Clone, PartialEq)]
struct ResolvedArguments {
    call_arguments: String,
    tetraplets: Vec<Vec<SecurityTetraplet>>,
}

impl<'i> ResolvedCall<'i> {
    /// Build `ResolvedCall` from `Call` by transforming `PeerPart` & `FunctionPart` into `ResolvedTriplet`.
    pub(super) fn new(raw_call: &Call<'i>, exec_ctx: &ExecutionCtx<'i>) -> ExecutionResult<Self> {
        let triplet = Triplet::try_from(&raw_call.peer_part, &raw_call.function_part)?;
        let triplet = triplet.resolve(exec_ctx)?;
        let triplet = Rc::new(triplet);

        Ok(Self {
            triplet,
            function_arg_paths: raw_call.args.clone(),
            output: raw_call.output.clone(),
        })
    }

    /// Executes resolved instruction, updates contexts based on a execution_step result.
    pub(super) fn execute(&self, exec_ctx: &mut ExecutionCtx<'i>, trace_ctx: &mut TraceHandler) -> ExecutionResult<()> {
        let should_execute = self.prepare_current_executed_state(exec_ctx, trace_ctx)?;
        if !should_execute {
            return Ok(());
        }

        // call can be executed only on peers with such peer_id
        if self.triplet.peer_pk.as_str() != exec_ctx.current_peer_id.as_str() {
            set_remote_call_result(self.triplet.peer_pk.clone(), exec_ctx, trace_ctx);
            return Ok(());
        }

        let ResolvedArguments {
            call_arguments,
            tetraplets,
        } = self.resolve_args(exec_ctx)?;

        let serialized_tetraplets = serde_json::to_string(&tetraplets).expect("default serializer shouldn't fail");

        let service_result = unsafe {
            crate::build_targets::call_service(
                &self.triplet.service_id,
                &self.triplet.function_name,
                &call_arguments,
                &serialized_tetraplets,
            )
        };

        self.update_state_with_service_result(service_result, exec_ctx, trace_ctx)
    }

    fn update_state_with_service_result(
        &self,
        service_result: CallServiceResult,
        exec_ctx: &mut ExecutionCtx<'i>,
        trace_ctx: &mut TraceHandler,
    ) -> ExecutionResult<()> {
        use ExecutionError::CallServiceResultDeError as DeError;

        // check that service call succeeded
        let service_result = handle_service_error(service_result, trace_ctx)?;

        let result: JValue = serde_json::from_str(&service_result.result).map_err(|e| DeError(service_result, e))?;
        let result = Rc::new(result);

        let trace_pos = trace_ctx.trace_pos();

        // TODO: refactor this scheme with passing and obtaining generation here in next PR
        let executed_result = ResolvedCallResult::new(result.clone(), self.triplet.clone(), trace_pos);
        let generation = set_local_call_result(executed_result, Generation::Last, &self.output, exec_ctx)?;
        let new_call_result = CallResult::Executed(result, generation);
        trace_ctx.meet_call_end(new_call_result);

        Ok(())
    }

    pub(super) fn as_triplet(&self) -> Rc<ResolvedTriplet> {
        self.triplet.clone()
    }

    /// Determine whether this call should be really called and adjust prev executed trace accordingly.
    fn prepare_current_executed_state(
        &self,
        exec_ctx: &mut ExecutionCtx<'i>,
        trace_ctx: &mut TraceHandler,
    ) -> ExecutionResult<bool> {
        let (call_result, trace_pos) = match trace_ctx.meet_call_start(&self.output)? {
            MergerCallResult::CallResult { value, trace_pos } => (value, trace_pos),
            MergerCallResult::Empty => return Ok(true),
        };

        handle_prev_state(&self.triplet, &self.output, call_result, trace_pos, exec_ctx, trace_ctx)
    }

    /// Prepare arguments of this call instruction by resolving and preparing their security tetraplets.
    fn resolve_args(&self, exec_ctx: &ExecutionCtx<'i>) -> ExecutionResult<ResolvedArguments> {
        use crate::execution_step::utils::resolve_to_args;

        let function_args = self.function_arg_paths.iter();
        let mut call_arguments = Vec::new();
        let mut tetraplets = Vec::new();
        for instruction_value in function_args {
            let (arg, tetraplet) = resolve_to_args(instruction_value, exec_ctx)?;
            call_arguments.push(arg);
            tetraplets.push(tetraplet);
        }

        let call_arguments = JValue::Array(call_arguments);
        let call_arguments = call_arguments.to_string();

        let resolved_arguments = ResolvedArguments {
            call_arguments,
            tetraplets,
        };

        Ok(resolved_arguments)
    }
}

fn handle_service_error(
    service_result: CallServiceResult,
    trace_ctx: &mut TraceHandler,
) -> ExecutionResult<CallServiceResult> {
    use CallResult::CallServiceFailed;

    if service_result.ret_code == CALL_SERVICE_SUCCESS {
        return Ok(service_result);
    }

    let error_message = Rc::new(service_result.result);
    let error = ExecutionError::LocalServiceError(service_result.ret_code, error_message.clone());
    let error = Rc::new(error);

    trace_ctx.meet_call_end(CallServiceFailed(service_result.ret_code, error_message));

    Err(error)
}
