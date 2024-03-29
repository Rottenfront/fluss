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

//! wayland platform support

// TODO: Remove this and fix the non-Send/Sync Arc issues
#![allow(clippy::arc_with_non_send_sync)]

pub mod application;
pub mod clipboard;
mod display;
pub mod error;
mod events;
pub mod keyboard;
pub mod menu;
mod outputs;
pub mod pointers;
pub mod screen;
pub mod surfaces;
pub mod window;

/// Little enum to make it clearer what some return values mean.
#[derive(Copy, Clone)]
enum Changed {
    Changed,
    Unchanged,
}

impl Changed {
    fn is_changed(self) -> bool {
        matches!(self, Changed::Changed)
    }
}
