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

mod traits;

use super::LexerError;
use super::LexerResult;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Token<'input> {
    OpenRoundBracket,
    CloseRoundBracket,
    OpenSquareBracket,
    CloseSquareBracket,

    StringLiteral(&'input str),
    Alphanumeric(&'input str),
    Stream(&'input str),
    VariableWithJsonPath(Variable<'input>, &'input str, bool),
    Number(Number),
    Boolean(bool),

    InitPeerId,
    LastError(LastErrorPath),

    Call,
    Seq,
    Par,
    Null,
    Fold,
    Xor,
    Next,
    Match,
    MisMatch,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum Variable<'input> {
    Scalar(&'input str),
    Stream(&'input str),
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum LastErrorPath {
    // %last_error%.instruction
    Instruction,
    // %last_error%.msg
    Message,
    // %last_error%.peer_id
    PeerId,
    // %last_error%
    None,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Number {
    Int(i64),
    Float(f64),
}

pub(crate) enum UnparsedNumber<'input> {
    // raw value and starting pos
    Int(&'input str, usize),
    Float(&'input str, usize),
}
