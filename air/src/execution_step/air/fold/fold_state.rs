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

use super::Instruction;
use super::IterableValue;
use super::ResolvedCallResult;

use std::collections::HashMap;
use std::rc::Rc;

pub(crate) struct FoldState<'i> {
    pub(crate) iterable: IterableValue,
    pub(crate) iterable_type: IterableType,
    pub(crate) instr_head: Rc<Instruction<'i>>,
    // map of met variables inside this (not any inner) fold block with their initial values
    pub(crate) met_variables: HashMap<&'i str, ResolvedCallResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IterableType {
    Scalar,
    Stream(Rc<String>),
}

impl<'i> FoldState<'i> {
    pub(crate) fn from_iterable(
        iterable: IterableValue,
        iterable_type: IterableType,
        instr_head: Rc<Instruction<'i>>,
    ) -> Self {
        Self {
            iterable,
            iterable_type,
            instr_head,
            met_variables: HashMap::new(),
        }
    }
}
