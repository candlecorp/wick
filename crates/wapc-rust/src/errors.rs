// Copyright 2015-2019 Capital One Services, LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Library-specific error types and utility functions

use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

pub fn new(kind: ErrorKind) -> Error {
    Error(Box::new(kind))
}

#[derive(Debug)]
pub enum ErrorKind {
    NoSuchFunction(String),
    IO(std::io::Error),
    WasmMisc(String),
    HostCallFailure(Box<dyn StdError + Sync + Send>),
    GuestCallFailure(String),
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    pub fn into_kind(self) -> ErrorKind {
        *self.0
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self.0 {
            ErrorKind::NoSuchFunction(_) => "No such function in Wasm module",
            ErrorKind::IO(_) => "I/O error",
            ErrorKind::WasmMisc(_) => "WebAssembly failure",
            ErrorKind::HostCallFailure(_) => "Error occurred during host call",
            ErrorKind::GuestCallFailure(_) => "Guest call failure",
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self.0 {
            ErrorKind::NoSuchFunction(_) => None,
            ErrorKind::IO(ref err) => Some(err),
            ErrorKind::WasmMisc(_) => None,
            ErrorKind::HostCallFailure(_) => None,
            ErrorKind::GuestCallFailure(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorKind::NoSuchFunction(ref fname) => {
                write!(f, "No such function in Wasm module: {}", fname)
            }
            ErrorKind::IO(ref err) => write!(f, "I/O error: {}", err),
            ErrorKind::WasmMisc(ref err) => write!(f, "WebAssembly error: {}", err),
            ErrorKind::HostCallFailure(ref err) => {
                write!(f, "Error occurred during host call: {}", err)
            }
            ErrorKind::GuestCallFailure(ref reason) => write!(f, "Guest call failure: {}", reason),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Error {
        Error(Box::new(ErrorKind::IO(source)))
    }
}

#[cfg(test)]
mod tests {
    #[allow(dead_code)]
    fn assert_sync_send<T: Send + Sync>() {}
    const _: fn() = || assert_sync_send::<super::Error>();
}
