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

use super::*;

mod _core {
    pub use aoide_core::track::release::Release;
}

///////////////////////////////////////////////////////////////////////
// Release
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    #[serde(rename = "own", skip_serializing_if = "Option::is_none")]
    released_by: Option<String>,

    #[serde(rename = "dat", skip_serializing_if = "Option::is_none")]
    released_at: Option<DateOrDateTime>,

    #[serde(rename = "cpy", skip_serializing_if = "Option::is_none")]
    copyright: Option<String>,

    #[serde(rename = "lic", skip_serializing_if = "Vec::is_empty", default)]
    licenses: Vec<String>,
}

impl From<_core::Release> for Release {
    fn from(from: _core::Release) -> Self {
        let _core::Release {
            released_at,
            released_by,
            copyright,
            licenses,
        } = from;
        Self {
            released_at: released_at.map(Into::into),
            released_by,
            copyright,
            licenses,
        }
    }
}

impl From<Release> for _core::Release {
    fn from(from: Release) -> Self {
        let Release {
            released_at,
            released_by,
            copyright,
            licenses,
        } = from;
        Self {
            released_at: released_at.map(Into::into),
            released_by,
            copyright,
            licenses,
        }
    }
}
