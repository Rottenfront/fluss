// Copyright 2019 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Errors at the application shell level.

use std::{fmt, sync::Arc};

use crate::backend::error as backend;

/// Shell errors.
#[derive(Debug, Clone)]
pub enum Error {
    /// The Application instance has already been created.
    ApplicationAlreadyExists,
    /// Tried to use the application after it had been dropped.
    ApplicationDropped,
    /// The window has already been destroyed.
    WindowDropped,
    /// Platform specific error.
    Platform(backend::Error),
    /// Other miscellaneous error.
    Other(Arc<anyhow::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::ApplicationAlreadyExists => {
                write!(f, "An application instance has already been created.")
            }
            Error::ApplicationDropped => {
                write!(
                    f,
                    "The application this operation requires has been dropped."
                )
            }
            Error::Platform(err) => fmt::Display::fmt(err, f),
            Error::WindowDropped => write!(f, "The window has already been destroyed."),
            Error::Other(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<anyhow::Error> for Error {
    fn from(src: anyhow::Error) -> Error {
        Error::Other(Arc::new(src))
    }
}

impl From<backend::Error> for Error {
    fn from(src: backend::Error) -> Error {
        Error::Platform(src)
    }
}
