// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![deny(missing_debug_implementations, missing_copy_implementations)]

///////////////////////////////////////////////////////////////////////
/// External Crates
///////////////////////////////////////////////////////////////////////
//
extern crate base64;

extern crate chrono;

#[macro_use]
extern crate failure;

extern crate log;

extern crate mime;

extern crate rand;

extern crate ring;

#[macro_use]
extern crate serde;

#[cfg(test)]
extern crate mime_guess;

#[cfg(test)]
extern crate serde_json;

///////////////////////////////////////////////////////////////////////
/// Public Modules
///////////////////////////////////////////////////////////////////////
//

#[allow(clippy::trivially_copy_pass_by_ref)]
pub mod domain;