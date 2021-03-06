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

use air_test_utils::call_vm;
use air_test_utils::create_avm;
use air_test_utils::set_variables_call_service;
use air_test_utils::unit_call_service;
use air_test_utils::ExecutionTrace;

use serde_json::json;

#[test]
fn dont_wait_on_json_path() {
    let status = json!({
        "err_msg": "",
        "is_authenticated": 1,
        "ret_code": 0,
    });

    let msg = String::from(r#""some message""#);

    let variables = maplit::hashmap!(
        "msg".to_string() => msg,
        "status".to_string() => status.to_string(),
    );

    let set_variables_call_service = set_variables_call_service(variables);

    let set_variable_peer_id = "set_variable";
    let mut set_variable_vm = create_avm(set_variables_call_service, set_variable_peer_id);

    let local_peer_id = "local_peer_id";
    let mut local_vm = create_avm(unit_call_service(), local_peer_id);

    let script = format!(
        r#"
        (seq
            (seq
                (call "{0}" ("" "") ["msg"] msg)
                (call "{0}" ("" "") ["status"] status)
            )
            (seq
                (call "{1}" ("op" "identity") [])
                (seq
                    (call "{1}" ("history" "add") [msg status.$.is_authenticated!] auth_result)
                    (call %init_peer_id% ("returnService" "run") [auth_result])
                )
            )
        )
    "#,
        set_variable_peer_id, local_peer_id
    );

    let init_peer_id = "asd";
    let res = call_vm!(set_variable_vm, init_peer_id, &script, "", "");
    let res = call_vm!(local_vm, init_peer_id, script, "", res.data);

    assert_eq!(res.next_peer_pks, vec![init_peer_id.to_string()]);
}

#[test]
fn wait_on_stream_json_path_by_id() {
    let local_peer_id = "local_peer_id";
    let mut local_vm = create_avm(unit_call_service(), local_peer_id);

    let non_join_stream_script = format!(
        r#"
    (par
        (call "{0}" ("" "") [] $status)
        (call "{0}" ("history" "add") [$status.$[0]!])
     )"#,
        local_peer_id
    );

    let res = call_vm!(local_vm, "", non_join_stream_script, "", "");
    let trace: ExecutionTrace = serde_json::from_slice(&res.data).expect("should be valid json");

    assert_eq!(res.ret_code, 0);
    assert_eq!(trace.len(), 3);

    let join_stream_script = format!(
        r#"
    (par
        (call "{0}" ("" "") [] $status)
        (call "{0}" ("history" "add") [$status.$[1]!]) ; $status stream here has only one value
     )"#,
        local_peer_id
    );

    let res = call_vm!(local_vm, "", join_stream_script, "", "");
    let trace: ExecutionTrace = serde_json::from_slice(&res.data).expect("should be valid json");

    assert_eq!(res.ret_code, 0);
    assert_eq!(trace.len(), 2); // par and the first call emit traces, second call doesn't
}

#[test]
fn dont_wait_on_json_path_on_scalars() {
    let array = json!([1, 2, 3, 4, 5]);

    let object = json!({
        "err_msg": "",
        "is_authenticated": 1,
        "ret_code": 0,
    });

    let variables = maplit::hashmap!(
        "array".to_string() => array.to_string(),
        "object".to_string() => object.to_string(),
    );

    let set_variables_call_service = set_variables_call_service(variables);

    let set_variable_peer_id = "set_variable";
    let mut set_variable_vm = create_avm(set_variables_call_service, set_variable_peer_id);

    let array_consumer_peer_id = "array_consumer_peer_id";
    let mut array_consumer = create_avm(unit_call_service(), array_consumer_peer_id);

    let object_consumer_peer_id = "object_consumer_peer_id";
    let mut object_consumer = create_avm(unit_call_service(), object_consumer_peer_id);

    let script = format!(
        r#"
        (seq
            (seq
                (call "{0}" ("" "") ["array"] array)
                (call "{0}" ("" "") ["object"] object)
            )
            (par
                (call "{1}" ("" "") [array.$.[5]!] auth_result)
                (call "{2}" ("" "") [object.$.non_exist_path])
            )
        )
    "#,
        set_variable_peer_id, array_consumer_peer_id, object_consumer_peer_id,
    );

    let init_peer_id = "asd";
    let res = call_vm!(set_variable_vm, init_peer_id, &script, "", "");
    let array_res = call_vm!(array_consumer, init_peer_id, &script, "", res.data.clone());
    assert_eq!(array_res.ret_code, 1006);
    assert_eq!(
        array_res.error_message,
        r#"variable with path '$.[5]' not found in '[1,2,3,4,5]' with an error: 'json value not set'"#
    );

    let object_res = call_vm!(object_consumer, init_peer_id, script, "", res.data);
    assert_eq!(object_res.ret_code, 1006);
    assert_eq!(
        object_res.error_message,
        r#"variable with path '$.non_exist_path' not found in '{"err_msg":"","is_authenticated":1,"ret_code":0}' with an error: 'json value not set'"#
    );
}
