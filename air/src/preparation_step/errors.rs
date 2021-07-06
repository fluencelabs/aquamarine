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

use serde_json::Error as SerdeJsonError;
use thiserror::Error as ThisError;

use std::env::VarError;

/// Errors happened during the interpreter preparation_step step.
#[derive(Debug, ThisError)]
pub enum PreparationError {
    /// Error occurred while parsing AIR script
    #[error("air can't be parsed:\n{0}")]
    AIRParseError(String),

    /// Errors occurred on executed trace deserialization.
    #[error("an error occurred while executed trace deserialization on {1:?}:\n {0:?}")]
    DataDeError(SerdeJsonError, Vec<u8>),

    /// Error occurred while getting current peer id.
    #[error("current peer id can't be obtained: {0:?}")]
    CurrentPeerIdEnvError(VarError),
}

impl PreparationError {
    pub(crate) fn to_error_code(&self) -> u32 {
        use PreparationError::*;

        match self {
            AIRParseError(_) => 1,
            DataDeError(..) => 2,
            CurrentPeerIdEnvError(_) => 3,
        }
    }
}
