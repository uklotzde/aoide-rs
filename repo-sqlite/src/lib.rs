// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#![deny(missing_debug_implementations)]
#![warn(rust_2018_idioms)]

// The following workaround is need to avoid cluttering the code with
// #[cfg_attr(feature = "diesel", ...)] to specify custom diesel
// attributes.
#[macro_use]
extern crate diesel;

// Workaround for using the embed_migrations!() macro in tests.
#[cfg(test)]
#[macro_use]
extern crate diesel_migrations;

use diesel::prelude::*;

pub mod collection;
pub mod playlist;
pub mod track;
pub mod util;
