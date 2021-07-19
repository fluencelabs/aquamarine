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

use super::Iterable;
use super::IterableItem;
use crate::foldable_next;
use crate::foldable_prev;
use crate::JValue;
use crate::SecurityTetraplet;

/// Used for iterating over a result of applied to a JValue json path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IterableJsonPathResult {
    pub(crate) jvalues: Vec<JValue>,
    // consider adding index for each tetraplet
    pub(crate) tetraplet: SecurityTetraplet,
    pub(crate) cursor: usize,
}

impl IterableJsonPathResult {
    pub(crate) fn init(jvalues: Vec<JValue>, tetraplet: SecurityTetraplet) -> Self {
        Self {
            jvalues,
            tetraplet,
            cursor: 0,
        }
    }
}

impl<'ctx> Iterable<'ctx> for IterableJsonPathResult {
    type Item = IterableItem<'ctx>;

    fn next(&mut self) -> bool {
        foldable_next!(self, self.jvalues.len())
    }

    fn prev(&mut self) -> bool {
        foldable_prev!(self)
    }

    fn peek(&'ctx self) -> Option<Self::Item> {
        if self.jvalues.is_empty() {
            return None;
        }

        let jvalue = &self.jvalues[self.cursor];
        let result = IterableItem::RefRef((jvalue, &self.tetraplet, 0));

        Some(result)
    }

    fn len(&self) -> usize {
        self.jvalues.len()
    }
}
