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

use serde_derive::Deserialize;
use serde_derive::Serialize;

/// arg path: head.$(json_path)
/// result:   head
/// This file contains defines of some things similar both for FCE and browser targets.

pub(crate) type Result<T> = std::result::Result<T, AquamarineError>;
pub(crate) type AquaData = std::collections::HashMap<String, JValue>;
pub(crate) type JValue = serde_json::Value;
pub(crate) use crate::errors::AquamarineError;
pub(crate) use crate::stepper_outcome::StepperOutcome;

pub(crate) const CALL_SERVICE_SUCCESS: i32 = 0;

#[fluence::fce]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallServiceResult {
    pub ret_code: i32,
    pub result: String,
}
