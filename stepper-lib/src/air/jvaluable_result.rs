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

use crate::air::fold::FoldableResult;
use crate::ExecutedCallResult;
use crate::JValue;
use crate::Result;
use crate::SecurityTetraplet;

use std::borrow::Cow;
use std::rc::Rc;

pub(crate) trait JValuableResult {
    fn apply_json_path(&self, json_path: &str) -> Result<Vec<&JValue>>;

    fn apply_json_path_with_tetraplets(&self, json_path: &str) -> Result<(Vec<&JValue>, Vec<SecurityTetraplet>)>;

    fn as_jvalue(&self) -> Cow<'_, JValue>;

    fn into_jvalue(self: Box<Self>) -> JValue;

    fn as_tetraplets(&self) -> Vec<SecurityTetraplet>;
}

impl JValuableResult for (JValue, SecurityTetraplet) {
    fn apply_json_path(&self, json_path: &str) -> Result<Vec<&JValue>> {
        use crate::AquamarineError::VariableNotInJsonPath as JsonPathError;
        use jsonpath_lib::select;

        let selected_jvalues =
            select(&self.0, json_path).map_err(|e| JsonPathError(self.0.clone(), String::from(json_path), e))?;
        Ok(selected_jvalues)
    }

    fn apply_json_path_with_tetraplets(&self, json_path: &str) -> Result<(Vec<&JValue>, Vec<SecurityTetraplet>)> {
        use crate::AquamarineError::VariableNotInJsonPath as JsonPathError;
        use jsonpath_lib::select;

        let selected_jvalues =
            select(&self.0, json_path).map_err(|e| JsonPathError(self.0.clone(), String::from(json_path), e))?;
        Ok((selected_jvalues, vec![self.1.clone()]))
    }

    fn as_jvalue(&self) -> Cow<'_, JValue> {
        Cow::Borrowed(&self.0)
    }

    fn into_jvalue(self: Box<Self>) -> JValue {
        self.0
    }

    fn as_tetraplets(&self) -> Vec<SecurityTetraplet> {
        // this clone is needed because of rust-sdk allows passing arguments only by value
        vec![self.1.clone()]
    }
}

impl JValuableResult for (&JValue, &SecurityTetraplet) {
    fn apply_json_path(&self, json_path: &str) -> Result<Vec<&JValue>> {
        use crate::AquamarineError::VariableNotInJsonPath as JsonPathError;
        use jsonpath_lib::select;

        let selected_jvalues =
            select(&self.0, json_path).map_err(|e| JsonPathError(self.0.clone(), String::from(json_path), e))?;
        Ok(selected_jvalues)
    }

    fn apply_json_path_with_tetraplets(&self, json_path: &str) -> Result<(Vec<&JValue>, Vec<SecurityTetraplet>)> {
        use crate::AquamarineError::VariableNotInJsonPath as JsonPathError;
        use jsonpath_lib::select;

        let selected_jvalues =
            select(&self.0, json_path).map_err(|e| JsonPathError(self.0.clone(), String::from(json_path), e))?;
        Ok((selected_jvalues, vec![self.1.clone()]))
    }

    fn as_jvalue(&self) -> Cow<'_, JValue> {
        Cow::Borrowed(&self.0)
    }

    fn into_jvalue(self: Box<Self>) -> JValue {
        self.0.clone()
    }

    fn as_tetraplets(&self) -> Vec<SecurityTetraplet> {
        // this clone is needed because of rust-sdk allows passing arguments only by value
        vec![self.1.clone()]
    }
}

impl<'ctx> JValuableResult for FoldableResult<'ctx> {
    fn apply_json_path(&self, json_path: &str) -> Result<Vec<&JValue>> {
        use crate::AquamarineError::VariableNotInJsonPath as JsonPathError;
        use jsonpath_lib::select;
        use std::ops::Deref;
        use FoldableResult::*;

        let jvalue = match self {
            Ref((jvalue, _)) => jvalue.deref(),
            Raw((jvalue, _)) => jvalue,
        };

        let selected_jvalues =
            select(jvalue, json_path).map_err(|e| JsonPathError(jvalue.clone(), String::from(json_path), e))?;
        Ok(selected_jvalues)
    }

    fn apply_json_path_with_tetraplets(&self, json_path: &str) -> Result<(Vec<&JValue>, Vec<SecurityTetraplet>)> {
        use crate::AquamarineError::VariableNotInJsonPath as JsonPathError;
        use jsonpath_lib::select;
        use std::ops::Deref;
        use FoldableResult::*;

        let (jvalue, tetraplet) = match self {
            Ref((jvalue, tetraplet)) => (jvalue.deref(), tetraplet.deref()),
            Raw((jvalue, tetraplet)) => (*jvalue, *tetraplet),
        };

        let selected_jvalues =
            select(jvalue, json_path).map_err(|e| JsonPathError(jvalue.clone(), String::from(json_path), e))?;
        Ok((selected_jvalues, vec![tetraplet.clone()]))
    }

    fn as_jvalue(&self) -> Cow<'_, JValue> {
        use FoldableResult::*;

        match self {
            Ref((jvalue, _)) => {
                use std::ops::Deref;
                let jvalue = jvalue.deref().clone();
                Cow::Owned(jvalue)
            }
            Raw((jvalue, _)) => Cow::Borrowed(jvalue),
        }
    }

    fn into_jvalue(self: Box<Self>) -> JValue {
        use std::ops::Deref;
        use FoldableResult::*;

        match *self {
            Ref((jvalue, _)) => jvalue.deref().clone(),
            Raw((jvalue, _)) => jvalue.clone(),
        }
    }

    fn as_tetraplets(&self) -> Vec<SecurityTetraplet> {
        use std::ops::Deref;
        use FoldableResult::*;

        // these clones is needed because of rust-sdk allows passing arguments only by value
        match self {
            Ref((_, tetraplet)) => {
                let tetraplet = tetraplet.deref().clone();
                vec![tetraplet]
            }
            Raw((_, tetraplet)) => vec![(*tetraplet).clone()],
        }
    }
}

impl JValuableResult for Rc<ExecutedCallResult> {
    fn apply_json_path(&self, json_path: &str) -> Result<Vec<&JValue>> {
        use crate::AquamarineError::VariableNotInJsonPath as JsonPathError;
        use jsonpath_lib::select;

        let selected_jvalues = select(&self.result, json_path)
            .map_err(|e| JsonPathError(self.result.clone(), String::from(json_path), e))?;
        Ok(selected_jvalues)
    }

    fn apply_json_path_with_tetraplets(&self, json_path: &str) -> Result<(Vec<&JValue>, Vec<SecurityTetraplet>)> {
        use crate::AquamarineError::VariableNotInJsonPath as JsonPathError;
        use jsonpath_lib::select;

        let selected_jvalues = select(&self.result, json_path)
            .map_err(|e| JsonPathError(self.result.clone(), String::from(json_path), e))?;
        Ok((selected_jvalues, vec![self.tetraplet.clone()]))
    }

    fn as_jvalue(&self) -> Cow<'_, JValue> {
        Cow::Borrowed(&self.result)
    }

    fn into_jvalue(self: Box<Self>) -> JValue {
        self.result.clone()
    }

    fn as_tetraplets(&self) -> Vec<SecurityTetraplet> {
        // this clone is needed because of rust-sdk allows passing arguments only by value
        vec![self.tetraplet.clone()]
    }
}

impl JValuableResult for std::cell::Ref<'_, Vec<Rc<ExecutedCallResult>>> {
    fn apply_json_path(&self, json_path: &str) -> Result<Vec<&JValue>> {
        use jsonpath_lib::select_with_iter;

        let (selected_values, _) = select_with_iter(self.iter().map(|r| &r.result), json_path).unwrap();

        Ok(selected_values)
    }

    fn apply_json_path_with_tetraplets(&self, json_path: &str) -> Result<(Vec<&JValue>, Vec<SecurityTetraplet>)> {
        use jsonpath_lib::select_with_iter;

        let (selected_values, tetraplet_indices) = select_with_iter(self.iter().map(|r| &r.result), json_path).unwrap();
        let tetraplets = tetraplet_indices
            .into_iter()
            // this cloned is needed because of rust-sdk allows passing arguments only by value
            .map(|id| self[id].tetraplet.clone())
            .collect::<Vec<_>>();

        Ok((selected_values, tetraplets))
    }

    fn as_jvalue(&self) -> Cow<'_, JValue> {
        let jvalue_array = self.iter().map(|r| r.result.clone()).collect::<Vec<_>>();
        Cow::Owned(JValue::Array(jvalue_array))
    }

    fn into_jvalue(self: Box<Self>) -> JValue {
        // this cloned is needed because of rust-sdk allows passing arguments only by value
        let jvalue_array = self.iter().map(|r| r.result.clone()).collect::<Vec<_>>();
        JValue::Array(jvalue_array)
    }

    fn as_tetraplets(&self) -> Vec<SecurityTetraplet> {
        self.iter()
            // this cloned is needed because of rust-sdk allows passing arguments only by value
            .map(|r| r.tetraplet.clone())
            .collect::<Vec<_>>()
    }
}