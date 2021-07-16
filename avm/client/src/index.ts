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

import { toByteArray } from 'base64-js';
import { return_current_peer_id, return_call_service_result, getStringFromWasm0, free, invoke, ast } from './wrapper';
import wasmBs64 from './wasm';
import { __wbg_callserviceimpl, __wbg_getcurrentpeeridimpl } from './importObject';

export type LogLevel = 'info' | 'trace' | 'debug' | 'info' | 'warn' | 'error' | 'off';

export type LogFunction = (level: LogLevel, message: string) => void;

export interface CallServiceResult {
    ret_code: number;
    result: string;
}

export interface ResolvedTriplet {
    peer_pk: string;
    service_id: string;
    function_name: string;
}

export interface SecurityTetraplet extends ResolvedTriplet {
    json_path: string;
}

export type ParticleHandler = (
    serviceId: string,
    fnName: string,
    args: any[],
    tetraplets: SecurityTetraplet[][],
) => CallServiceResult;

type Exports = any;
type Instance = any;
type ExportValue = any;

type LogImport = {
    log_utf8_string: (level: any, target: any, offset: any, size: any) => void;
};

type ImportObject = {
    './air_interpreter_client_bg.js': {
        // fn call_service_impl(service_id: String, fn_name: String, args: String, security_tetraplets: String) -> String;
        // prettier-ignore
        [__wbg_callserviceimpl]: (arg0: any, arg1: any, arg2: any, arg3: any, arg4: any, arg5: any, arg6: any, arg7: any, arg8: any, ) => void;
        [__wbg_getcurrentpeeridimpl]: (arg0: any) => void;
        __wbindgen_throw: (arg: any) => void;
    };
    host: LogImport;
};

class HostImportsConfig {
    exports: Exports | undefined;
    newImportObject: () => ImportObject;

    constructor(create: (cfg: HostImportsConfig) => ImportObject) {
        this.exports = undefined;
        this.newImportObject = () => create(this);
    }

    setExports(exports: Exports) {
        this.exports = exports;
    }
}

const interpreter_wasm = toByteArray(wasmBs64);

/// Instantiates WebAssembly runtime with AIR interpreter module
async function interpreterInstance(cfg: HostImportsConfig, logFunction: LogFunction): Promise<Instance> {
    /// Create host imports that use module exports internally
    let imports = cfg.newImportObject();

    /// Instantiate interpreter
    let interpreter_module = await WebAssembly.compile(interpreter_wasm);
    let instance: Instance = await WebAssembly.instantiate(interpreter_module, imports);

    /// Set exports, so host imports can use them
    cfg.setExports(instance.exports);

    /// Trigger interpreter initialization (i.e., call main function)
    call_export(instance.exports.main, logFunction);

    return instance;
}

/// If export is a function, call it. Otherwise log a warning.
/// NOTE: any here is unavoidable, see Function interface definition
function call_export(f: ExportValue, logFunction: LogFunction): any {
    if (typeof f === 'function') {
        return f();
    } else {
        logFunction('error', `can't call export ${f}: it is not a function, but ${typeof f}`);
    }
}

function log_import(cfg: HostImportsConfig, logFunction: LogFunction): LogImport {
    return {
        log_utf8_string: (level: any, target: any, offset: any, size: any) => {
            let wasm = cfg.exports;

            try {
                let str = getStringFromWasm0(wasm, offset, size);
                let levelStr: LogLevel;
                switch (level) {
                    case 1:
                        levelStr = 'error';
                        break;
                    case 2:
                        levelStr = 'warn';
                        break;
                    case 3:
                        levelStr = 'info';
                        break;
                    case 4:
                        levelStr = 'debug';
                        break;
                    case 6:
                        levelStr = 'trace';
                        break;
                    default:
                        return;
                }
                logFunction(levelStr, str);
            } finally {
            }
        },
    };
}

/// Returns import object that describes host functions called by AIR interpreter
function newImportObject(
    particleHandler: ParticleHandler,
    cfg: HostImportsConfig,
    peerId: string,
    logFunction: LogFunction,
): ImportObject {
    return {
        // __wbg_callserviceimpl_c0ca292e3c8c0c97 this is a function generated by bindgen. Could be changed.
        // If so, an error with a new name will be occurred after wasm initialization.
        './air_interpreter_client_bg.js': {
            // prettier-ignore
            [__wbg_callserviceimpl]: (arg0: any, arg1: any, arg2: any, arg3: any, arg4: any, arg5: any, arg6: any, arg7: any, arg8: any) => {
                let wasm = cfg.exports;
                let serviceId = getStringFromWasm0(wasm, arg1, arg2);
                let fnName = getStringFromWasm0(wasm, arg3, arg4);
                let args = getStringFromWasm0(wasm, arg5, arg6);
                let tetraplets = getStringFromWasm0(wasm, arg7, arg8);
                
                let argsObject;
                let tetrapletsObject: SecurityTetraplet[][];
                let serviceResult;
                try {
                    argsObject = JSON.parse(args);
                    if (!Array.isArray(argsObject)) {
                        throw new Error('args is not an array');
                    }

                    tetrapletsObject = JSON.parse(tetraplets);
                    serviceResult = particleHandler(serviceId, fnName, argsObject, tetrapletsObject);
                } catch (err) {
                    logFunction('error', 'Cannot parse arguments: ' + JSON.stringify(err));
                    serviceResult = {
                        result: JSON.stringify('Cannot parse arguments: ' + JSON.stringify(err)),
                        ret_code: 1,
                    };
                }

                let resultStr = JSON.stringify(serviceResult);
                return_call_service_result(wasm, resultStr, arg0);
            },
            [__wbg_getcurrentpeeridimpl]: (arg0: any) => {
                let wasm = cfg.exports;
                return_current_peer_id(wasm, peerId, arg0);
            },
            __wbindgen_throw: (arg: any) => {
                throw new Error(`wbindgen throws: ${JSON.stringify(arg)}`);
            },
        },
        host: log_import(cfg, logFunction),
    };
}

export class AirInterpreter {
    private wasmWrapper;
    private logLevel: LogLevel;

    constructor(wasmWrapper) {
        this.wasmWrapper = wasmWrapper;
    }

    static async create(
        particleHandler: ParticleHandler,
        peerId: string,
        logLevel: LogLevel,
        logFunction: LogFunction,
    ) {
        const cfg = new HostImportsConfig((cfg) => {
            return newImportObject(particleHandler, cfg, peerId, logFunction);
        });

        const instance = await interpreterInstance(cfg, logFunction);
        const res = new AirInterpreter(instance);
        res.logLevel = logLevel;
        return res;
    }

    invoke(init_peer_id: string, script: string, prev_data: Uint8Array, data: Uint8Array): string {
        return invoke(this.wasmWrapper.exports, init_peer_id, script, prev_data, data, this.logLevel);
    }

    parseAir(script: string): string {
        return ast(this.wasmWrapper.exports, script);
    }
}
