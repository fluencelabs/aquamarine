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

// use pretty_assertions::assert_eq;
use serde_json::json;

use air_test_utils::*;

type JValue = serde_json::Value;

use std::collections::HashMap;

#[test]
fn data_merge() {
    use executed_state::*;

    let set_variable_id = "set_variable";
    let vm_1_id = "A";
    let vm_2_id = "B";

    let mut set_variable = create_avm(
        set_variable_call_service(json!([vm_1_id, vm_2_id]).to_string()),
        set_variable_id,
    );
    let mut vm1 = create_avm(return_string_call_service(vm_1_id), vm_1_id);
    let mut vm2 = create_avm(return_string_call_service(vm_2_id), vm_2_id);

    let script = r#"
        (seq
            (call "set_variable" ("neighborhood" "") [] neighborhood)
            (seq
                (seq
                    (fold neighborhood i
                        (par
                            (call i ("add_provider" "") [] $void)
                            (next i)
                        )
                    )
                    (fold neighborhood i
                        (par
                            (call i ("get_providers" "") [] $providers)
                            (next i)
                        )
                    )
                )
                (seq
                    (call "A" ("identity" "") [] $void)
                    (call "B" ("" "") [] none)
                )
            )
        )
        "#;

    // little hack here with init_peer_id to execute the first call from both VMs
    let result_0 = checked_call_vm!(set_variable, "", script, "", "");
    let result_1 = checked_call_vm!(vm1, "", script, "", result_0.data.clone());
    let result_2 = checked_call_vm!(vm2, "", script, "", result_0.data);
    let result_3 = checked_call_vm!(vm1, "", script, result_1.data.clone(), result_2.data.clone());
    let result_4 = checked_call_vm!(vm2, "", script, result_1.data.clone(), result_2.data.clone());

    let actual_trace_1 = trace_from_result(&result_1);

    let expected_trace_1 = vec![
        scalar_string_array(vec!["A", "B"]),
        par(1, 2),
        stream_string(vm_1_id, 0),
        par(1, 0),
        request_sent_by(set_variable_id),
        par(1, 2),
        stream_string(vm_1_id, 0),
        par(1, 0),
        request_sent_by(vm_1_id),
        stream_string(vm_1_id, 0),
        request_sent_by(vm_1_id),
    ];

    assert_eq!(actual_trace_1, expected_trace_1);
    assert_eq!(result_1.next_peer_pks, vec![String::from("B")]);

    let actual_trace_2 = trace_from_result(&result_2);

    let expected_trace_2 = vec![
        scalar_string_array(vec!["A", "B"]),
        par(1, 2),
        request_sent_by(set_variable_id),
        par(1, 0),
        stream_string(vm_2_id, 0),
        par(1, 2),
        request_sent_by(vm_2_id),
        par(1, 0),
        stream_string(vm_2_id, 0),
        request_sent_by(vm_2_id),
    ];

    assert_eq!(actual_trace_2, expected_trace_2);
    assert_eq!(result_2.next_peer_pks, vec![String::from("A")]);

    let actual_trace_3 = trace_from_result(&result_3);

    let expected_trace_3 = vec![
        scalar_string_array(vec!["A", "B"]),
        par(1, 2),
        stream_string(vm_1_id, 0),
        par(1, 0),
        stream_string(vm_2_id, 1),
        par(1, 2),
        stream_string(vm_1_id, 0),
        par(1, 0),
        stream_string(vm_2_id, 1),
        stream_string(vm_1_id, 0),
        request_sent_by(vm_1_id),
    ];

    assert_eq!(actual_trace_3, expected_trace_3);
    assert!(result_3.next_peer_pks.is_empty());

    let actual_trace_4 = trace_from_result(&result_4);

    let expected_trace_4 = vec![
        scalar_string_array(vec!["A", "B"]),
        par(1, 2),
        stream_string(vm_1_id, 0),
        par(1, 0),
        stream_string(vm_2_id, 1),
        par(1, 2),
        stream_string(vm_1_id, 0),
        par(1, 0),
        stream_string(vm_2_id, 1),
        stream_string(vm_1_id, 0),
        scalar_string(vm_2_id),
    ];

    assert_eq!(actual_trace_4, expected_trace_4);
    assert!(result_4.next_peer_pks.is_empty());
}

#[test]
fn acc_merge() {
    let neighborhood_call_service: CallServiceClosure = Box::new(|args| -> Option<IValue> {
        let args_count = match &args.function_args[1] {
            IValue::String(str) => str,
            _ => unreachable!(),
        };

        let args_count = (args_count.as_bytes()[0] - b'0') as usize;

        let args_json = match &args.function_args[2] {
            IValue::String(str) => str,
            _ => unreachable!(),
        };

        let args: Vec<JValue> = serde_json::from_str(args_json).expect("valid json");
        let args = match &args[0] {
            JValue::Array(args) => args,
            _ => unreachable!(),
        };

        assert_eq!(args.len(), args_count);

        Some(IValue::Record(
            NEVec::new(vec![IValue::S32(0), IValue::String(json!(args).to_string())]).unwrap(),
        ))
    });

    let mut vm1 = create_avm(set_variable_call_service(r#""peer_id""#), "A");
    let mut vm2 = create_avm(neighborhood_call_service, "B");

    let script = String::from(
        r#"
        (seq 
            (call "A" ("add_provider" "") [] $void)
            (seq 
                (call "A" ("add_provider" "") [] $void)
                (seq 
                    (call "A" ("get_providers" "") [] $providers)
                    (seq 
                        (call "A" ("get_providers" "") [] $providers)
                        (seq 
                            (call "B" ("" "2") [$providers] $void)
                            (call "B" ("" "3") [$void] $void)
                        )
                    )
                )
            )
        )
        "#,
    );

    let result = checked_call_vm!(vm1, "asd", script.clone(), "", "");
    checked_call_vm!(vm2, "asd", script, "", result.data);
}

#[test]
fn fold_merge__() {
    use std::ops::Deref;

    let set_variable_vm_id = "set_variable";
    let local_vm_id = "local_vm";

    let variables = maplit::hashmap! {
        "stream_1".to_string() => json!(["peer_0", "peer_1", "peer_2", "peer_3"]).to_string(),
        "stream_2".to_string() => json!(["peer_4", "peer_5", "peer_6"]).to_string(),
    };
    let mut set_variable_vm = create_avm(set_variables_call_service(variables), set_variable_vm_id);

    let script = format!(
        include_str!("./scripts/inner_folds_v1.clj"),
        set_variable_vm_id, local_vm_id
    );

    let set_variable_result = checked_call_vm!(set_variable_vm, "", &script, "", "");

    let mut local_vms = Vec::with_capacity(7);
    let mut local_vms_results = Vec::with_capacity(7);
    for vm_id in 0..7 {
        let peer_id = format!("peer_{}", vm_id);
        let mut vm = create_avm(echo_string_call_service(), peer_id);
        let result = checked_call_vm!(
            vm,
            "",
            &script,
            set_variable_result.data.clone(),
            set_variable_result.data.clone()
        );

        local_vms.push(vm);
        local_vms_results.push(result);
    }

    let mut local_vm = create_avm(echo_string_call_service(), local_vm_id);
    let result_1 = checked_call_vm!(local_vm, "", &script, "", local_vms_results[0].data.clone());
    let result_2 = checked_call_vm!(
        local_vm,
        "",
        &script,
        result_1.data.clone(),
        local_vms_results[3].data.clone()
    );
    let result_3 = checked_call_vm!(
        local_vm,
        "",
        &script,
        result_2.data.clone(),
        local_vms_results[4].data.clone()
    );
    let result_4 = checked_call_vm!(
        local_vm,
        "",
        &script,
        result_3.data.clone(),
        local_vms_results[5].data.clone()
    );

    let result_5 = checked_call_vm!(
        local_vm,
        "",
        &script,
        result_4.data.clone(),
        local_vms_results[1].data.clone()
    );

    let result_6 = checked_call_vm!(
        local_vm,
        "",
        &script,
        result_5.data.clone(),
        local_vms_results[2].data.clone()
    );

    let result_7 = checked_call_vm!(
        local_vm,
        "",
        &script,
        result_6.data.clone(),
        local_vms_results[6].data.clone()
    );

    let data = InterpreterData::try_from_slice(&result_7.data).expect("data should be well-formed");
    let stream_1_generations = data
        .streams
        .get("$stream_1")
        .expect("$stream_1 should presence in data");
    let stream_2_generations = data
        .streams
        .get("$stream_2")
        .expect("$stream_2 should presence in data");

    assert_eq!(*stream_1_generations, 4);
    assert_eq!(*stream_2_generations, 3);

    let mut fold_states_count = 0;
    let mut calls_count = HashMap::new();

    for state in data.trace.iter() {
        if matches!(state, ExecutedState::Fold(_)) {
            fold_states_count += 1;
        }

        if let ExecutedState::Fold(fold) = state {
            for subtrace_lore in fold.0.iter() {
                let value_pos = subtrace_lore.value_pos as usize;
                if let ExecutedState::Call(CallResult::Executed(value, _)) = &data.trace[value_pos] {
                    if let JValue::String(var_name) = value.deref() {
                        let current_count: usize = calls_count.get(var_name).map(|v| *v).unwrap_or_default();
                        calls_count.insert(var_name, current_count + 1);
                    }
                }
            }
        }
    }

    // $stream_1 contains 4 generation each with 2 elements, it means that inner fold for $stream_2
    // runs 2*4 times and produces 2*4 fold states, and 1 state comes from fold for $stream_1
    assert_eq!(fold_states_count, 2 * 4 + 1);

    for (call_result, call_count) in calls_count {
        if call_result.as_str() < "peer_4" {
            assert_eq!(call_count, 2);
        } else {
            assert_eq!(call_count, 16);
        }
    }
}
