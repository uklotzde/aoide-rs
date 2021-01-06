// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use crate::usecases::playlists::entries as uc;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum PatchOperation {
    Append {
        entries: Vec<Entry>,
    },
    Prepend {
        entries: Vec<Entry>,
    },
    Insert {
        before: usize,
        entries: Vec<Entry>,
    },
    Move {
        start: usize,
        end: usize,
        delta: isize,
    },
    Remove {
        start: usize,
        end: usize,
    },
    Clear,
    Reverse,
    Shuffle,
}

impl From<PatchOperation> for uc::PatchOperation {
    fn from(from: PatchOperation) -> Self {
        use PatchOperation::*;
        match from {
            Append { entries } => Self::Append {
                entries: entries.into_iter().map(Into::into).collect(),
            },
            Prepend { entries } => Self::Prepend {
                entries: entries.into_iter().map(Into::into).collect(),
            },
            Insert { before, entries } => Self::Insert {
                before,
                entries: entries.into_iter().map(Into::into).collect(),
            },
            Move { start, end, delta } => Self::Move {
                range: start..end,
                delta,
            },
            Remove { start, end } => Self::Remove { range: start..end },
            Clear => Self::Clear,
            Reverse => Self::Reverse,
            Shuffle => Self::Shuffle,
        }
    }
}

pub type RequestBody = Vec<PatchOperation>;

pub type ResponseBody = EntityWithEntriesSummary;

pub fn handle_request(
    pooled_connection: &SqlitePooledConnection,
    uid: EntityUid,
    query_params: EntityRevQueryParams,
    request_body: RequestBody,
) -> RepoResult<ResponseBody> {
    let EntityRevQueryParams { rev } = query_params;
    let entity_header = _core::EntityHeader {
        uid,
        rev: rev.into(),
    };
    uc::patch(
        pooled_connection,
        &entity_header,
        request_body.into_iter().map(Into::into),
    )
    .map(|(_, entity_with_entries_summary)| entity_with_entries_summary)
}

///////////////////////////////////////////////////////////////////////