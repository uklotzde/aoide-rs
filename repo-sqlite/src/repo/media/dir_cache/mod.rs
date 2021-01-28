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

use crate::{
    db::media_dir_cache::{models::*, schema::*},
    prelude::*,
};

use aoide_core::util::clock::DateTime;

use aoide_repo::{collection::RecordId as CollectionId, media::dir_cache::*};

use num_traits::{FromPrimitive as _, ToPrimitive as _};

#[derive(QueryableByName)]
struct StatusCountRow {
    #[sql_type = "diesel::sql_types::SmallInt"]
    status: i16,

    #[sql_type = "diesel::sql_types::BigInt"]
    count: i64,
}

impl<'db> Repo for crate::prelude::Connection<'db> {
    fn media_dir_cache_update_entries_status(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri_prefix: &str,
        old_status: Option<EntryStatus>,
        new_status: EntryStatus,
    ) -> RepoResult<usize> {
        let target = media_dir_cache::table
            .filter(media_dir_cache::collection_id.eq(RowId::from(collection_id)))
            .filter(diesel::dsl::sql(&format!(
                "substr(uri,1,{})='{}'",
                uri_prefix.len(),
                escape_single_quotes(uri_prefix),
            )));
        let mut query = diesel::update(target)
            .set((
                media_dir_cache::row_updated_ms.eq(updated_at.timestamp_millis()),
                media_dir_cache::status.eq(new_status.to_i16().expect("new_status")),
            ))
            .into_boxed();
        if let Some(old_status) = old_status {
            query =
                query.filter(media_dir_cache::status.eq(old_status.to_i16().expect("old_status")));
        }
        query.execute(self.as_ref()).map_err(repo_error)
    }

    fn media_dir_cache_delete_entries(
        &self,
        collection_id: CollectionId,
        uri_prefix: &str,
        status: Option<EntryStatus>,
    ) -> RepoResult<usize> {
        let target = media_dir_cache::table
            .filter(media_dir_cache::collection_id.eq(RowId::from(collection_id)))
            .filter(diesel::dsl::sql(&format!(
                "substr(uri,1,{})='{}'",
                uri_prefix.len(),
                escape_single_quotes(uri_prefix),
            )))
            .filter(
                media_dir_cache::status.eq(EntryStatus::Orphaned.to_i16().expect("not updated")),
            );
        let mut query = diesel::delete(target).into_boxed();
        if let Some(status) = status {
            query = query.filter(media_dir_cache::status.eq(status.to_i16().expect("status")));
        }
        query.execute(self.as_ref()).map_err(repo_error)
    }

    fn media_dir_cache_update_entry_digest(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri: &str,
        digest: &EntryDigest,
    ) -> RepoResult<UpdateOutcome> {
        // Try to mark outdated entry as current if digest is unchanged (most likely)
        let target = media_dir_cache::table
            .filter(media_dir_cache::collection_id.eq(RowId::from(collection_id)))
            .filter(media_dir_cache::uri.eq(uri))
            .filter(media_dir_cache::digest.eq(&digest[..]))
            // Filtering by EntryStatus::Outdated allows to safely trigger a rescan even
            // if entries that have previously been marked as added or modified are still
            // pending for subsequent processing, e.g. (re-)importing their metadata.
            // Those entries will finally be skipped (see below).
            .filter(
                media_dir_cache::status
                    .eq(EntryStatus::Outdated.to_i16().expect("outdated"))
                    .or(media_dir_cache::status
                        .eq(EntryStatus::Orphaned.to_i16().expect("orphaned"))),
            );
        let query = diesel::update(target).set((
            media_dir_cache::row_updated_ms.eq(updated_at.timestamp_millis()),
            media_dir_cache::status.eq(EntryStatus::Current.to_i16().expect("current")),
        ));
        let rows_affected = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected > 0 {
            return Ok(UpdateOutcome::Current);
        }
        // Try to mark existing entry (with any status) as modified if digest has changed (less likely)
        let target = media_dir_cache::table
            .filter(media_dir_cache::collection_id.eq(RowId::from(collection_id)))
            .filter(media_dir_cache::uri.eq(uri))
            .filter(media_dir_cache::digest.ne(&digest[..]));
        let query = diesel::update(target).set((
            media_dir_cache::row_updated_ms.eq(updated_at.timestamp_millis()),
            media_dir_cache::status.eq(EntryStatus::Modified.to_i16().expect("modified")),
            media_dir_cache::digest.eq(&digest[..]),
        ));
        let rows_affected = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected > 0 {
            return Ok(UpdateOutcome::Updated);
        }
        // Try to add a new entry (least likely)
        let insertable =
            InsertableRecord::bind(updated_at, collection_id, uri, EntryStatus::Added, digest);
        let query = diesel::insert_or_ignore_into(media_dir_cache::table).values(&insertable);
        let rows_affected = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected > 0 {
            return Ok(UpdateOutcome::Inserted);
        }
        // Skip entries that have previously been marked as either added or
        // modified if their digest didn't change.
        Ok(UpdateOutcome::Skipped)
    }

    fn media_dir_cache_reset_entry_status_to_current(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri: &str,
        digest: &EntryDigest,
    ) -> RepoResult<bool> {
        let target = media_dir_cache::table
            .filter(media_dir_cache::collection_id.eq(RowId::from(collection_id)))
            .filter(media_dir_cache::uri.eq(uri))
            .filter(media_dir_cache::digest.eq(&digest[..]));
        let query = diesel::update(target).set((
            media_dir_cache::row_updated_ms.eq(updated_at.timestamp_millis()),
            media_dir_cache::status.eq(EntryStatus::Current.to_i16().expect("current")),
        ));
        let rows_affected = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        Ok(rows_affected > 0)
    }

    fn media_dir_cache_load_entry_status_by_uri(
        &self,
        collection_id: CollectionId,
        uri: &str,
    ) -> RepoResult<EntryStatus> {
        media_dir_cache::table
            .select(media_dir_cache::status)
            .filter(media_dir_cache::collection_id.eq(RowId::from(collection_id)))
            .filter(media_dir_cache::uri.eq(uri))
            .first::<i16>(self.as_ref())
            .map_err(repo_error)
            .map(|val| EntryStatus::from_i16(val).expect("EntryStatus"))
    }

    fn media_dir_cache_update_load_entries_aggregate_status(
        &self,
        collection_id: CollectionId,
        uri_prefix: &str,
    ) -> RepoResult<AggregateStatus> {
        // TODO: Remove with type-safe query when group_by() is available
        /*
        media_dir_cache::table
            .select((media_dir_cache::status, diesel::dsl::count_star))
            .filter(media_dir_cache::collection_id.eq(RowId::from(collection_id)))
            .filter(diesel::dsl::sql(&format!(
                "substr(uri,1,{})='{}'",
                uri_prefix.len(),
                escape_single_quotes(uri_prefix),
            )))
            // TODO: Replace with group_by() when available
            .filter(diesel::dsl::sql("TRUE GROUP BY status ORDER BY status"))
            .load::<(i16, usize)>(self.as_ref())
        */
        let sql = format!(
            "SELECT status, COUNT(*) as count \
        FROM media_dir_cache \
        WHERE collection_id={collection_id} AND \
        substr(uri,1,{uri_prefix_len})='{escaped_uri_prefix}' \
        GROUP BY status",
            collection_id = RowId::from(collection_id),
            uri_prefix_len = uri_prefix.len(),
            escaped_uri_prefix = escape_single_quotes(uri_prefix),
        );
        diesel::dsl::sql_query(sql)
            .load::<StatusCountRow>(self.as_ref())
            .map_err(repo_error)
            .map(|v| {
                v.into_iter()
                    .fold(AggregateStatus::default(), |mut aggregate_status, row| {
                        let StatusCountRow { status, count } = row;
                        let status = EntryStatus::from_i16(status).expect("EntryStatus");
                        let count = (count as u64) as usize;
                        match status {
                            EntryStatus::Current => {
                                debug_assert_eq!(aggregate_status.current, 0);
                                aggregate_status.current = count;
                            }
                            EntryStatus::Outdated => {
                                debug_assert_eq!(aggregate_status.outdated, 0);
                                aggregate_status.outdated = count;
                            }
                            EntryStatus::Added => {
                                debug_assert_eq!(aggregate_status.added, 0);
                                aggregate_status.added = count;
                            }
                            EntryStatus::Modified => {
                                debug_assert_eq!(aggregate_status.modified, 0);
                                aggregate_status.modified = count;
                            }
                            EntryStatus::Orphaned => {
                                debug_assert_eq!(aggregate_status.orphaned, 0);
                                aggregate_status.orphaned = count;
                            }
                        }
                        aggregate_status
                    })
            })
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
