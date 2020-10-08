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

#![warn(rust_2018_idioms)]
#![deny(
    dead_code,
    nonstandard_style,
    unused_imports,
    unused_mut,
    unused_variables,
    unused_unsafe,
    unreachable_patterns
)]

use aquamarine_vm::vec1::Vec1;
use aquamarine_vm::AquamarineVM;
use aquamarine_vm::AquamarineVMConfig;
use aquamarine_vm::HostExportedFunc;
use aquamarine_vm::HostImportDescriptor;
use aquamarine_vm::IType;
use aquamarine_vm::IValue;

use std::path::PathBuf;

pub fn create_aqua_vm(call_service: HostExportedFunc) -> AquamarineVM {
    let call_service_descriptor = HostImportDescriptor {
        host_exported_func: call_service,
        argument_types: vec![IType::String, IType::String, IType::String],
        output_type: Some(IType::Record(0)),
        error_handler: None,
    };

    let config = AquamarineVMConfig {
        aquamarine_wasm_path: PathBuf::from("../target/wasm32-wasi/debug/aquamarine.wasm"),
        call_service: call_service_descriptor,
        current_peer_id: String::from("test_peer_id"),
    };

    AquamarineVM::new(config).expect("vm should be created")
}

pub fn unit_call_service() -> HostExportedFunc {
    Box::new(|_, _| -> Option<IValue> {
        Some(IValue::Record(
            Vec1::new(vec![
                IValue::S32(0),
                IValue::String(String::from("\"test\"")),
            ])
            .unwrap(),
        ))
    })
}

pub fn echo_string_call_service() -> HostExportedFunc {
    Box::new(|_, args| -> Option<IValue> {
        let arg = match &args[2] {
            IValue::String(str) => str,
            _ => unreachable!(),
        };

        let arg: Vec<String> = serde_json::from_str(arg).unwrap();

        Some(IValue::Record(
            Vec1::new(vec![
                IValue::S32(0),
                IValue::String(format!("\"{}\"", arg[0])),
            ])
            .unwrap(),
        ))
    })
}

pub fn echo_number_call_service() -> HostExportedFunc {
    Box::new(|_, args| -> Option<IValue> {
        let arg = match &args[2] {
            IValue::String(str) => str,
            _ => unreachable!(),
        };

        let arg: Vec<String> = serde_json::from_str(arg).unwrap();

        Some(IValue::Record(
            Vec1::new(vec![IValue::S32(0), IValue::String(arg[0].clone())]).unwrap(),
        ))
    })
}