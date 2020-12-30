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

use super::EvidenceState;

use serde::Error as SerdeJsonError;

use std::env::VarError;
use std::error::Error;

/// Errors happened during the stepper preparation step.
#[derive(Debug)]
pub(crate) enum PreparationError {
    /// Error occurred while parsing AIR script
    AIRParseError(String),

    /// Errors occurred on call evidence deserialization.
    CallEvidenceDeError(SerdeJsonError, Vec<u8>),

    /// Indicates that environment variable with current name doesn't set.
    CurrentPeerIdEnvError(VarError, String),

    /// Errors occurred while merging previous and current data.
    StateMergingError(DataMergingError),
}

/// Errors arose out of merging previous data with a new.
#[derive(Debug)]
pub(crate) enum DataMergingError {
    /// Errors occurred when previous and current evidence states are incompatible.
    IncompatibleEvidenceStates(EvidenceState, EvidenceState),

    /// Errors occurred when previous and current call results are incompatible.
    IncompatibleCallResults(CallResult, CallResult),

    /// Errors occurred when evidence path contains less elements then corresponding Par has.
    EvidencePathTooSmall(usize, usize),
}

impl Error for PreparationError {}
impl Error for DataMergingError {}

impl std::fmt::Display for PreparationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Self::*;

        match self {
            AIRParseError(err_msg) => write!(f, "aqua script can't be parsed:\n{}", err),
            CallEvidenceDeError(serde_error, evidence_path) => {
                let print_error = move |path| {
                    write!(
                        f,
                        "an error occurred while call evidence path deserialization on \"{:?}\": {:?}",
                        path, serde_error
                    )
                };

                let path = match String::from_utf8(evidence_path) {
                    Ok(str) => print_error(str),
                    Err(e) => print_error(e.into_bytes()),
                };
            }
            CurrentPeerIdEnvError(err, env_name) => write!(
                f,
                "the environment variable with name \"{}\" can't be obtained: {:?}",
                env_name, err
            ),
            StateMergingError(err) => write!(f, "{}", err),
        }
    }
}

impl std::fmt::Display for DataMergingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Self::*;

        match self {
            IncompatibleEvidenceStates(prev_state, current_state) => write!(
                f,
                "previous and current data have incompatible states: {:?} {:?}",
                prev_state, current_state
            ),
            IncompatibleCallResults(prev_call_result, current_call_result) => write!(
                f,
                "previous and current call results are incompatible: {:?} {:?}",
                prev_call_result, current_call_result
            ),
            EvidencePathTooSmall(actual_count, expected_count) => write!(
                f,
                "evidence path remains {} elements, but {} requires by Par",
                actual_count, desired_count
            ),
        }
    }
}

impl From<std::convert::Infallible> for PreparationError {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}

impl From<std::convert::Infallible> for DataMergingError {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}
