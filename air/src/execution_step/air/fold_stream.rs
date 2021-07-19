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

use super::fold::*;
use super::fold_scalar::fold;
use super::ExecutableInstruction;
use super::ExecutionCtx;
use super::ExecutionResult;
use super::TraceHandler;
use crate::log_instruction;

use air_parser::ast::FoldStream;

impl<'i> ExecutableInstruction<'i> for FoldStream<'i> {
    fn execute(&self, exec_ctx: &mut ExecutionCtx<'i>, trace_ctx: &mut TraceHandler) -> ExecutionResult<()> {
        log_instruction!(fold, exec_ctx, trace_ctx);

        let iterables = match construct_stream_iterable_value(self.stream_name, exec_ctx)? {
            FoldIterableStream::Empty => return Ok(()),
            FoldIterableStream::Stream(iterables) => iterables,
        };

        trace_ctx.meet_fold_start(self.id.to_string())?;

        for iterable in iterables {
            let value = match iterable.peek() {
                Some(value) => value,
                // it's ok, because some generation level of a stream on some point inside execution
                // flow could contain zero values
                None => continue,
            };

            let value = value.as_value_and_pos();
            trace_ctx.meet_iteration_start(self.id.as_str(), &value)?;
            fold(
                iterable,
                IterableType::Stream(self.id.clone()),
                self.iterator,
                self.instruction.clone(),
                exec_ctx,
                trace_ctx,
            )?;
            trace_ctx.meet_generation_end(self.id.as_str())?;
        }

        trace_ctx.meet_fold_end(self.id.as_str())?;

        Ok(())
    }
}
