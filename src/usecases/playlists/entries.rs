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

use aoide_core::util::clock::DateTime;
use aoide_core_serde::playlist::EntityWithEntriesSummary;

use std::ops::Range;

///////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PatchOperation {
    Append { entries: Vec<Entry> },
    Prepend { entries: Vec<Entry> },
    Insert { before: usize, entries: Vec<Entry> },
    Move { range: Range<usize>, delta: isize },
    Remove { range: Range<usize> },
    Clear,
    Reverse,
    Shuffle,
}

pub fn patch(
    pooled_connection: &SqlitePooledConnection,
    entity_header: &EntityHeader,
    operations: impl IntoIterator<Item = PatchOperation>,
) -> RepoResult<(RecordHeader, EntityWithEntriesSummary)> {
    let updated_at = DateTime::now_utc();
    let db = SqliteConnection::new(&*pooled_connection);
    Ok(db.transaction::<_, DieselRepoError, _>(|| {
        let (record_header, _next_rev) =
            db.touch_playlist_entity_revision(&entity_header, updated_at)?;
        for operation in operations.into_iter() {
            use PatchOperation::*;
            match operation {
                Append { entries } => {
                    if entries.is_empty() {
                        continue;
                    }
                    db.append_playlist_entries(record_header.id, entries)?;
                }
                Prepend { entries } => {
                    if entries.is_empty() {
                        continue;
                    }
                    db.prepend_playlist_entries(record_header.id, entries)?;
                }
                Insert { before, entries } => {
                    if entries.is_empty() {
                        continue;
                    }
                    db.insert_playlist_entries(record_header.id, before, entries)?;
                }
                Move { range, delta } => {
                    if range.is_empty() || delta == 0 {
                        continue;
                    }
                    db.move_playlist_entries(record_header.id, &range, delta)?;
                }
                Remove { range } => {
                    if range.is_empty() {
                        continue;
                    }
                    db.remove_playlist_entries(record_header.id, &range)?;
                }
                Clear => {
                    db.clear_playlist_entries(record_header.id)?;
                }
                Reverse => {
                    db.reverse_playlist_entries(record_header.id)?;
                }
                Shuffle => {
                    db.shuffle_playlist_entries(record_header.id)?;
                }
            }
        }
        let (record_header, entity, entries_summary) =
            db.load_playlist_entity_with_entries_summary(record_header.id)?;
        debug_assert_eq!(_next_rev, entity.hdr.rev);
        Ok((record_header, (entity, entries_summary).into()))
    })?)
}