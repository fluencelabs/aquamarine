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

use super::Joinable;
use crate::build_targets::CallServiceResult;
use crate::contexts::execution::ResolvedCallResult;
use crate::contexts::execution_trace::ExecutedState;
use crate::contexts::execution_trace::ExecutionTrace;
use crate::JValue;

use jsonpath_lib::JsonPathError;
use serde_json::Error as SerdeJsonError;
use thiserror::Error as ThisError;

use std::rc::Rc;

/// Errors arised while executing AIR script.
#[derive(ThisError, Debug)]
pub(crate) enum ExecutionError {
    /// Errors occurred while parsing returned by call_service value.
    #[error("call_service result '{0}' can't be serialized or deserialized with an error: {1}")]
    CallServiceResultDeError(CallServiceResult, SerdeJsonError),

    /// Semantic errors in instructions.
    #[error("{0}")]
    InstructionError(String),

    /// An error is occurred while calling local service via call_service.
    #[error("Local service error, ret_code is {0}, error message is '{1}'")]
    LocalServiceError(i32, Rc<String>),

    /// Value for such name isn't presence in data.
    #[error("variable with name '{0}' isn't present in data")]
    VariableNotFound(String),

    /// Multiple values for such name found.
    #[error("multiple variables found for name '{0}' in data")]
    MultipleVariablesFound(String),

    /// An error occurred while trying to apply json path to this JValue.
    #[error("variable with path '{1}' not found in '{0}' with an error: '{2}'")]
    JValueJsonPathError(JValue, String, JsonPathError),

    /// An error occurred while trying to apply json path to this stream with JValue's.
    #[error("variable with path '{1}' not found in '{0:?}' with error: '{2}'")]
    JValueStreamJsonPathError(Vec<ResolvedCallResult>, String, JsonPathError),

    /// Provided JValue has incompatible with target type.
    #[error("expected JValue type '{1}', but got '{0}' JValue")]
    IncompatibleJValueType(JValue, &'static str),

    /// Provided AValue has incompatible with target type.
    #[error("expected AValue type '{1}', but got '{0}' AValue")]
    IncompatibleAValueType(String, String),

    /// Multiple values found for such json path.
    #[error("multiple variables found for this json path '{0}'")]
    MultipleValuesInJsonPath(String),

    /// Fold state wasn't found for such iterator name.
    #[error("fold state not found for this iterable '{0}'")]
    FoldStateNotFound(String),

    /// Multiple fold states found for such iterator name.
    #[error("multiple fold states found for iterable '{0}'")]
    MultipleFoldStates(String),

    /// Expected executed state of a different type.
    #[error(
        r#"
    ======================================================================
    Below is a crucial debug information, please send this to the Fluence Labs:

    invalid executed state error:
      current instruction: {instruction},
      expected state: {expected_state},
      actual_state: '{actual_state}',
      current_trace: '{current_trace:?}',
      new_trace: '{new_trace:?}',
      current_subtree_size: {current_subtree_size},

    ======================================================================
    "#
    )]
    InvalidExecutedState {
        instruction: String,
        expected_state: &'static str,
        actual_state: ExecutedState,
        current_trace: ExecutionTrace,
        new_trace: ExecutionTrace,
        current_subtree_size: usize,
    },

    /// Errors encountered while shadowing non-scalar values.
    #[error("variable with name '{0}' can't be shadowed, shadowing is supported only for scalar values")]
    ShadowingError(String),

    /// This error type is produced by a match to notify xor that compared values aren't equal.
    #[error("match is used without corresponding xor")]
    MatchWithoutXorError,

    /// This error type is produced by a mismatch to notify xor that compared values aren't equal.
    #[error("mismatch is used without corresponding xor")]
    MismatchWithoutXorError,

    /// This error type is produced by a mismatch to notify xor that compared values aren't equal.
    #[error("jvalue '{0}' can't be flattened, to be flattened a jvalue should have an array type and consist of zero or one values")]
    FlatteningError(JValue),

    /// Json path is applied to scalar that have inappropriate type.
    #[error(
        "json path can't be applied to scalar '{0}',\
    it could be applied only to streams and variables of array and object types"
    )]
    JsonPathVariableTypeError(JValue),
}

impl ExecutionError {
    pub(crate) fn to_error_code(&self) -> u32 {
        use ExecutionError::*;

        match self {
            CallServiceResultDeError(..) => 1,
            InstructionError(_) => 2,
            LocalServiceError(..) => 3,
            VariableNotFound(_) => 4,
            MultipleVariablesFound(_) => 5,
            JValueJsonPathError(..) => 6,
            JValueStreamJsonPathError(..) => 7,
            IncompatibleJValueType(..) => 8,
            IncompatibleAValueType(..) => 9,
            MultipleValuesInJsonPath(_) => 10,
            FoldStateNotFound(_) => 11,
            MultipleFoldStates(_) => 12,
            InvalidExecutedState { .. } => 13,
            ShadowingError(_) => 14,
            MatchWithoutXorError => 15,
            MismatchWithoutXorError => 16,
            FlatteningError(_) => 17,
            JsonPathVariableTypeError(_) => 18,
        }
    }
}

macro_rules! log_join {
    ($($args:tt)*) => {
        log::info!(target: crate::log_targets::JOIN_BEHAVIOUR, $($args)*)
    }
}

#[rustfmt::skip::macros(log_join)]
impl Joinable for ExecutionError {
    /// Returns true, if supplied error is related to variable not found errors type.
    /// Print log if this is joinable error type.
    fn is_joinable(&self) -> bool {
        use ExecutionError::*;

        match self {
            VariableNotFound(var_name) => {
                log_join!("  waiting for an argument with name '{}'", var_name);
                true
            }
            JValueStreamJsonPathError(stream, json_path, _) => {
                log_join!("  waiting for an argument with path '{}' on stream '{:?}'", json_path, stream);
                true
            }
            _ => false,
        }
    }
}

impl From<std::convert::Infallible> for ExecutionError {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}
