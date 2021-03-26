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

use super::ExecutionCtx;
use super::ExecutionError;
use super::ExecutionResult;
use super::ExecutionTraceCtx;
use crate::log_instruction;

use air_parser::ast::Xor;

impl<'i> super::ExecutableInstruction<'i> for Xor<'i> {
    fn execute(&self, exec_ctx: &mut ExecutionCtx<'i>, trace_ctx: &mut ExecutionTraceCtx) -> ExecutionResult<()> {
        log_instruction!(xor, exec_ctx, trace_ctx);

        exec_ctx.subtree_complete = true;
        match self.0.execute(exec_ctx, trace_ctx) {
            Err(e) if is_catchable_by_xor(&e) => {
                exec_ctx.subtree_complete = true;
                exec_ctx.last_error_could_be_set = true;
                log::warn!("xor caught an error: {}", e);

                self.1.execute(exec_ctx, trace_ctx)
            }
            res => res,
        }
    }
}

/// Returns true, if this execution error type should be caught by xor.
fn is_catchable_by_xor(exec_error: &ExecutionError) -> bool {
    // this type of errors related to invalid data and should treat as hard errors.
    !matches!(exec_error, ExecutionError::InvalidExecutedState(..))
}

#[cfg(test)]
mod tests {
    use aqua_test_utils::call_vm;
    use aqua_test_utils::create_aqua_vm;
    use aqua_test_utils::executed_state;
    use aqua_test_utils::fallible_call_service;
    use aqua_test_utils::ExecutionTrace;

    #[test]
    fn xor() {
        let local_peer_id = "local_peer_id";
        let fallible_service_id = String::from("service_id_1");
        let mut vm = create_aqua_vm(fallible_call_service(fallible_service_id), local_peer_id);

        let script = format!(
            r#"
            (xor
                (call "{0}" ("service_id_1" "local_fn_name") [] result_1)
                (call "{0}" ("service_id_2" "local_fn_name") [] result_2)
            )"#,
            local_peer_id,
        );

        let res = call_vm!(vm, "asd", script, "[]", "[]");
        let actual_trace: ExecutionTrace = serde_json::from_slice(&res.data).expect("should be valid json");
        let expected_call_result = executed_state::scalar_string("res");

        assert_eq!(actual_trace.len(), 2);
        assert_eq!(actual_trace[0], executed_state::service_failed(1, "error"));
        assert_eq!(actual_trace[1], expected_call_result);

        let script = format!(
            r#"
            (xor
                (call "{0}" ("service_id_2" "local_fn_name") [] result_1)
                (call "{0}" ("service_id_1" "local_fn_name") [] result_2)
            )"#,
            local_peer_id
        );

        let res = call_vm!(vm, "asd", script, "[]", "[]");
        let actual_trace: ExecutionTrace = serde_json::from_slice(&res.data).expect("should be valid json");

        assert_eq!(actual_trace.len(), 1);
        assert_eq!(actual_trace[0], expected_call_result);
    }

    #[test]
    fn xor_var_not_found() {
        use crate::contexts::execution_trace::CallResult::*;
        use crate::contexts::execution_trace::ExecutedState::*;
        use aqua_test_utils::echo_string_call_service;

        let local_peer_id = "local_peer_id";
        let mut vm = create_aqua_vm(echo_string_call_service(), local_peer_id);

        let script = format!(
            r#"
            (xor
                (par
                    (call "unknown_peer" ("service_id_1" "local_fn_name") [] lazy_defined_variable)
                    (call "{0}" ("service_id_1" "local_fn_name") [lazy_defined_variable] result)
                )
                (call "{0}" ("service_id_2" "local_fn_name") ["expected"] result)
            )"#,
            local_peer_id,
        );

        let res = call_vm!(vm, "asd", script, "[]", "[]");
        let actual_trace: ExecutionTrace = serde_json::from_slice(&res.data).expect("should be valid json");
        assert_eq!(actual_trace[0], Par(1, 0));
        assert_eq!(actual_trace[1], Call(RequestSentBy(String::from("local_peer_id"))));
    }

    #[test]
    fn xor_multiple_variables_found() {
        use aqua_test_utils::echo_string_call_service;

        let set_variables_peer_id = "set_variables_peer_id";
        let mut set_variables_vm = create_aqua_vm(echo_string_call_service(), set_variables_peer_id);

        let local_peer_id = "local_peer_id";
        let mut vm = create_aqua_vm(echo_string_call_service(), local_peer_id);

        let some_string = String::from("some_string");
        let expected_string = String::from("expected_string");
        let script = format!(
            r#"
            (seq
                (call "{0}" ("service_id_1" "local_fn_name") ["{2}"] result_1)
                (xor
                    (call "{1}" ("service_id_1" "local_fn_name") [""] result_1)
                    (call "{1}" ("service_id_2" "local_fn_name") ["{3}"] result_2)
                )
            )"#,
            set_variables_peer_id, local_peer_id, some_string, expected_string
        );

        let res = call_vm!(set_variables_vm, "asd", script.clone(), "[]", "[]");
        let res = call_vm!(vm, "asd", script, "[]", res.data);
        let actual_trace: ExecutionTrace = serde_json::from_slice(&res.data).expect("should be valid json");
        let some_string_call_result = executed_state::scalar_string(some_string);
        let expected_string_call_result = executed_state::scalar_string(expected_string);

        assert_eq!(actual_trace.len(), 2);
        assert_eq!(actual_trace[0], some_string_call_result);
        assert_eq!(actual_trace[1], expected_string_call_result);
    }

    #[test]
    fn xor_par() {
        use aqua_test_utils::executed_state::*;

        let fallible_service_id = String::from("service_id_1");
        let local_peer_id = "local_peer_id";
        let mut vm = create_aqua_vm(fallible_call_service(fallible_service_id.clone()), local_peer_id);

        let script = format!(
            r#"
            (xor
                (par
                    (seq
                        (call "{0}" ("service_id_2" "local_fn_name") [] result_1)
                        (call "{0}" ("service_id_2" "local_fn_name") [] result_2)
                    )
                    (par
                        (call "{0}" ("service_id_1" "local_fn_name") [] result_3)
                        (call "{0}" ("service_id_2" "local_fn_name") [] result_4)
                    )
                )
                (seq
                    (call "{0}" ("service_id_2" "local_fn_name") [] result_4)
                    (call "{0}" ("service_id_2" "local_fn_name") [] result_5)
                )
            )"#,
            local_peer_id
        );

        let result = call_vm!(vm, "asd", script.clone(), "[]", "[]");
        let actual_trace: ExecutionTrace = serde_json::from_slice(&result.data).expect("should be valid json");

        let res = String::from("res");

        let expected_trace = vec![
            par(2, 2),
            scalar_string(&res),
            scalar_string(&res),
            par(1, 0),
            service_failed(1, "error"),
            scalar_string(&res),
            scalar_string(&res),
        ];

        assert_eq!(actual_trace, expected_trace);

        let result = call_vm!(vm, "asd", script, "[]", result.data);
        let actual_trace: ExecutionTrace = serde_json::from_slice(&result.data).expect("should be valid json");
        assert_eq!(actual_trace, expected_trace);
    }

    #[test]
    fn last_error_with_xor() {
        use aqua_test_utils::echo_string_call_service;

        let faillible_peer_id = "failible_peer_id";
        let mut faillible_vm = create_aqua_vm(fallible_call_service("service_id_1"), faillible_peer_id);
        let local_peer_id = "local_peer_id";
        let mut vm = create_aqua_vm(echo_string_call_service(), local_peer_id);

        let script = format!(
            r#"
            (xor
                (call "{0}" ("service_id_1" "local_fn_name") [] result)
                (call "{1}" ("service_id_2" "local_fn_name") [%last_error%] result)
            )"#,
            faillible_peer_id, local_peer_id,
        );

        let res = call_vm!(faillible_vm, "asd", script.clone(), "", "");
        let res = call_vm!(vm, "asd", script, "", res.data);
        let actual_trace: ExecutionTrace = serde_json::from_slice(&res.data).expect("should be valid json");

        let expected_state = aqua_test_utils::executed_state::scalar_string("{\"error\":\"Local service error: ret_code is 1, error message is \'error\'\",\"instruction\":\"call \\\"failible_peer_id\\\" (\\\"service_id_1\\\" \\\"local_fn_name\\\") [] result\"}");

        assert_eq!(actual_trace[1], expected_state);
    }
}
