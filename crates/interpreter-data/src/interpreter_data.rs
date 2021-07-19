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

use super::ExecutedState;
use super::DATA_FORMAT_VERSION;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::ops::Deref;

pub type StreamGenerations = HashMap<String, u32>;
pub type ExecutionTrace = Vec<ExecutedState>;

/// The AIR interpreter could be considered as a function
/// f(prev_data: InterpreterData, current_data: InterpreterData, ... ) -> (result_data: InterpreterData, ...).
/// This function receives prev and current data and produces a result data. All these data
/// have the following format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpreterData {
    /// Trace of AIR execution, which contains executed call, par and fold states.
    pub trace: ExecutionTrace,

    /// Contains maximum generation for each stream. This info will be used while merging
    /// values in streams.
    pub streams: StreamGenerations,

    /// Version of this data format.
    pub version: semver::Version,
}

impl InterpreterData {
    pub fn new() -> Self {
        Self {
            trace: <_>::default(),
            streams: <_>::default(),
            version: DATA_FORMAT_VERSION.deref().clone(),
        }
    }

    pub fn from_execution_result(trace: ExecutionTrace, streams: StreamGenerations) -> Self {
        Self {
            trace,
            streams,
            version: DATA_FORMAT_VERSION.deref().clone(),
        }
    }

    /// Tries to de InterpreterData from slice according to the data version.
    pub fn try_from_slice(slice: &[u8]) -> Result<Self, serde_json::Error> {
        // treat empty slice as an empty interpreter data allows abstracting from
        // the internal format for empty data.
        if slice.is_empty() {
            return Ok(Self::default());
        }

        serde_json::from_slice(slice)
    }
}

impl Default for InterpreterData {
    fn default() -> Self {
        Self::new()
    }
}
