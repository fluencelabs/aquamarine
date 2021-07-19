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

use crate::execution_step::air::ExecutionResult;
use crate::execution_step::execution_context::ExecutionCtx;
use crate::execution_step::utils::get_variable_name;
use crate::execution_step::utils::resolve_to_jvaluable;
use crate::JValue;

use air_parser::ast;
use air_parser::ast::MatchableValue;

pub(crate) fn are_matchable_eq<'ctx>(
    left: &MatchableValue<'_>,
    right: &MatchableValue<'_>,
    exec_ctx: &'ctx ExecutionCtx<'_>,
) -> ExecutionResult<bool> {
    use MatchableValue::*;

    match (left, right) {
        (InitPeerId, InitPeerId) => Ok(true),
        (InitPeerId, matchable) => compare_matchable(
            matchable,
            exec_ctx,
            make_string_comparator(exec_ctx.init_peer_id.as_str()),
        ),
        (matchable, InitPeerId) => compare_matchable(
            matchable,
            exec_ctx,
            make_string_comparator(exec_ctx.init_peer_id.as_str()),
        ),

        (Literal(left_name), Literal(right_name)) => Ok(left_name == right_name),
        (Literal(value), matchable) => compare_matchable(matchable, exec_ctx, make_string_comparator(value)),
        (matchable, Literal(value)) => compare_matchable(matchable, exec_ctx, make_string_comparator(value)),

        (Boolean(value), matchable) => compare_matchable(matchable, exec_ctx, make_bool_comparator(value)),
        (matchable, Boolean(value)) => compare_matchable(matchable, exec_ctx, make_bool_comparator(value)),

        (Number(value), matchable) => compare_matchable(matchable, exec_ctx, make_number_comparator(value)),
        (matchable, Number(value)) => compare_matchable(matchable, exec_ctx, make_number_comparator(value)),

        (Variable(left_variable), Variable(right_variable)) => {
            let left_name = get_variable_name(left_variable);
            let left_jvaluable = resolve_to_jvaluable(left_name, exec_ctx)?;
            let left_value = left_jvaluable.as_jvalue();

            let right_name = get_variable_name(right_variable);
            let right_jvaluable = resolve_to_jvaluable(right_name, exec_ctx)?;
            let right_value = right_jvaluable.as_jvalue();

            Ok(left_value == right_value)
        }
        (
            JsonPath {
                variable: lv,
                path: lp,
                should_flatten: lsf,
            },
            JsonPath {
                variable: rv,
                path: rp,
                should_flatten: rsf,
            },
        ) => {
            // TODO: improve comparison
            if lsf != rsf {
                return Ok(false);
            }

            let left_name = get_variable_name(lv);
            let left_jvaluable = resolve_to_jvaluable(left_name, exec_ctx)?;
            let left_value = left_jvaluable.apply_json_path(lp)?;

            let right_name = get_variable_name(rv);
            let right_jvaluable = resolve_to_jvaluable(right_name, exec_ctx)?;
            let right_value = right_jvaluable.apply_json_path(rp)?;

            Ok(left_value == right_value)
        }
        _ => Ok(false),
    }
}

use std::borrow::Cow;
type Comparator<'a> = Box<dyn Fn(Cow<'_, JValue>) -> bool + 'a>;

fn compare_matchable<'ctx>(
    matchable: &MatchableValue<'_>,
    exec_ctx: &'ctx ExecutionCtx<'_>,
    comparator: Comparator<'ctx>,
) -> ExecutionResult<bool> {
    use MatchableValue::*;

    match matchable {
        InitPeerId => {
            let init_peer_id = exec_ctx.init_peer_id.clone();
            let jvalue = init_peer_id.into();
            Ok(comparator(Cow::Owned(jvalue)))
        }
        Literal(str) => {
            let jvalue = str.to_string().into();
            Ok(comparator(Cow::Owned(jvalue)))
        }
        Number(number) => {
            let jvalue = number.into();
            Ok(comparator(Cow::Owned(jvalue)))
        }
        Boolean(bool) => {
            let jvalue = (*bool).into();
            Ok(comparator(Cow::Owned(jvalue)))
        }
        Variable(variable) => {
            let name = get_variable_name(variable);
            let jvaluable = resolve_to_jvaluable(name, exec_ctx)?;
            let jvalue = jvaluable.as_jvalue();
            Ok(comparator(jvalue))
        }
        JsonPath {
            variable,
            path,
            should_flatten,
        } => {
            let var_name = get_variable_name(variable);
            let jvaluable = resolve_to_jvaluable(var_name, exec_ctx)?;
            let jvalues = jvaluable.apply_json_path(path)?;

            let jvalue = if *should_flatten {
                if jvalues.len() != 1 {
                    return Ok(false);
                }
                Cow::Borrowed(jvalues[0])
            } else {
                let jvalue = jvalues.into_iter().cloned().collect::<Vec<_>>();
                let jvalue = JValue::Array(jvalue);

                Cow::Owned(jvalue)
            };

            Ok(comparator(jvalue))
        }
    }
}

fn make_string_comparator(comparable_string: &str) -> Comparator<'_> {
    use std::ops::Deref;

    Box::new(move |jvalue: Cow<'_, JValue>| -> bool {
        match jvalue.deref() {
            JValue::String(value) => value == comparable_string,
            _ => false,
        }
    })
}

fn make_bool_comparator(comparable_bool: &bool) -> Comparator<'_> {
    use std::ops::Deref;

    let comparable_bool = *comparable_bool;
    Box::new(move |jvalue: Cow<'_, JValue>| -> bool {
        match jvalue.deref() {
            JValue::Bool(jvalue) => jvalue == &comparable_bool,
            _ => false,
        }
    })
}

fn make_number_comparator(comparable_number: &ast::Number) -> Comparator<'_> {
    use std::ops::Deref;

    let comparable_jvalue: JValue = comparable_number.into();
    Box::new(move |jvalue: Cow<'_, JValue>| -> bool { jvalue.deref() == &comparable_jvalue })
}
