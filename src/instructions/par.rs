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

use super::Instruction;
use crate::AquaData;
use crate::Result;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) struct Par(Box<Instruction>, Box<Instruction>);

impl super::ExecutableInstruction for Par {
    fn execute(self, data: &mut AquaData, next_peer_pks: &mut Vec<String>) -> Result<()> {
        log::info!("par called with data: {:?} and next_peer_pks: {:?}", data, next_peer_pks);

        self.0.execute(data, next_peer_pks)?;
        self.1.execute(data, next_peer_pks)?;

        Ok(())
    }
}
