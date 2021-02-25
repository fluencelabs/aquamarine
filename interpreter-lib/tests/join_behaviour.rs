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

use aqua_test_utils::call_vm;
use aqua_test_utils::create_aqua_vm;
use aqua_test_utils::set_variables_call_service;
use aqua_test_utils::unit_call_service;

use interpreter_lib::execution_trace::ExecutionTrace;

use serde_json::json;

#[test]
fn absent_json_path() {
    use log::LevelFilter::Info;

    let status = json!({
        "err_msg": "",
        "is_authorized": 1,
        "ret_code": 0,
    });

    let msg = String::from(r#""some message""#);

    let variables = maplit::hashmap!(
        "msg".to_string() => msg,
        "status".to_string() => status.to_string(),
    );

    let set_variables_call_service = set_variables_call_service(variables);

    let set_variable_peer_id = "set_variable";
    let mut set_variable_vm = create_aqua_vm(set_variables_call_service, set_variable_peer_id);

    let local_peer_id = "local_peer_id";
    let mut local_vm = create_aqua_vm(unit_call_service(), local_peer_id);

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
                    (call "{1}" ("history" "add") [msg status.$.is_authorized] auth_result)
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