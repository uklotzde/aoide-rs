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
    db::{
        media_source::{models::*, schema::*, subselect},
        track::schema::*,
    },
    prelude::*,
};

use aoide_core::{media::Source, util::clock::DateTime};

use aoide_repo::{collection::RecordId as CollectionId, media::source::*};

impl<'db> Repo for crate::prelude::Connection<'db> {
    fn resolve_media_source_id_synchronized_at_by_path(
        &self,
        collection_id: CollectionId,
        path: &str,
    ) -> RepoResult<(RecordId, Option<DateTime>)> {
        Ok(media_source::table
            .select((
                media_source::row_id,
                media_source::synchronized_at,
                media_source::synchronized_ms,
            ))
            .filter(media_source::collection_id.eq(RowId::from(collection_id)))
            .filter(media_source::path.eq(path))
            .first::<(RowId, Option<String>, Option<i64>)>(self.as_ref())
            .map(|(row_id, synchronized_at, synchronized_ms)| {
                (
                    row_id.into(),
                    parse_datetime_opt(synchronized_at.as_deref(), synchronized_ms),
                )
            })
            .map_err(repo_error)?)
    }

    fn resolve_media_source_ids_by_path_predicate(
        &self,
        collection_id: CollectionId,
        path_predicate: StringPredicateBorrowed<'_>,
    ) -> RepoResult<Vec<RecordId>> {
        media_source::table
            .select(media_source::row_id)
            // Reuse the tested subselect with reliable predicate filtering
            // even if it might be slightly less efficient! The query optimizer
            // should detect this.
            .filter(
                media_source::row_id.eq_any(subselect::filter_by_path_predicate(
                    collection_id,
                    path_predicate,
                )),
            )
            .load::<RowId>(self.as_ref())
            .map_err(repo_error)
            .map(|v| v.into_iter().map(RecordId::new).collect())
    }

    fn relocate_media_sources_by_path_prefix(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        old_path_prefix: &str,
        new_path_prefix: &str,
    ) -> RepoResult<usize> {
        let target = media_source::table
            .filter(media_source::collection_id.eq(RowId::from(collection_id)))
            .filter(diesel::dsl::sql(&format!(
                "substr(path,1,{})='{}'",
                old_path_prefix.len(),
                escape_single_quotes(old_path_prefix),
            )));
        diesel::update(target)
            .set((
                media_source::row_updated_ms.eq(updated_at.timestamp_millis()),
                media_source::path.eq(diesel::dsl::sql(&format!(
                    "'{}' || substr(path,{})",
                    escape_single_quotes(new_path_prefix),
                    old_path_prefix.len() + 1
                ))),
            ))
            .execute(self.as_ref())
            .map_err(repo_error)
    }

    fn purge_media_sources_by_path_predicate(
        &self,
        collection_id: CollectionId,
        path_predicate: StringPredicateBorrowed<'_>,
    ) -> RepoResult<usize> {
        // Reuse the tested subselect with reliable predicate filtering
        // even if it might be slightly less efficient! The query optimizer
        // should detect this.
        diesel::delete(media_source::table.filter(media_source::row_id.eq_any(
            subselect::filter_by_path_predicate(collection_id, path_predicate),
        )))
        .execute(self.as_ref())
        .map_err(repo_error)
    }

    fn insert_media_source(
        &self,
        created_at: DateTime,
        collection_id: CollectionId,
        created_source: &Source,
    ) -> RepoResult<RecordHeader> {
        let insertable = InsertableRecord::bind(created_at, collection_id, created_source);
        let query = diesel::insert_into(media_source::table).values(&insertable);
        let rows_affected: usize = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected == 1);
        let (id, _) = self
            .resolve_media_source_id_synchronized_at_by_path(collection_id, &created_source.path)?;
        Ok(RecordHeader {
            id,
            created_at,
            updated_at: created_at,
        })
    }

    fn update_media_source(
        &self,
        id: RecordId,
        updated_at: DateTime,
        updated_source: &Source,
    ) -> RepoResult<()> {
        let updatable = UpdatableRecord::bind(updated_at, updated_source);
        let target = media_source::table.filter(media_source::row_id.eq(RowId::from(id)));
        let query = diesel::update(target).set(&updatable);
        let rows_affected: usize = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn delete_media_source(&self, id: RecordId) -> RepoResult<()> {
        let target = media_source::table.filter(media_source::row_id.eq(RowId::from(id)));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn load_media_source(&self, id: RecordId) -> RepoResult<(RecordHeader, Source)> {
        media_source::table
            .filter(media_source::row_id.eq(RowId::from(id)))
            .first::<QueryableRecord>(self.as_ref())
            .map_err(repo_error)
            .map(Into::into)
    }

    fn load_media_source_by_path(
        &self,
        collection_id: CollectionId,
        path: &str,
    ) -> RepoResult<(RecordHeader, Source)> {
        media_source::table
            .filter(media_source::collection_id.eq(RowId::from(collection_id)))
            .filter(media_source::path.eq(path))
            .first::<QueryableRecord>(self.as_ref())
            .map_err(repo_error)
            .map(Into::into)
    }

    fn purge_orphaned_media_sources_from_collection(
        &self,
        collection_id: CollectionId,
    ) -> RepoResult<usize> {
        let target = media_source::table
            .filter(media_source::collection_id.eq(RowId::from(collection_id)))
            .filter(media_source::row_id.ne_all(track::table.select(track::media_source_id)));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_ref()).map_err(repo_error)?;
        Ok(rows_affected)
    }

    fn purge_orphaned_media_sources_from_all_collections(&self) -> RepoResult<usize> {
        let target = media_source::table
            .filter(media_source::row_id.ne_all(track::table.select(track::media_source_id)));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_ref()).map_err(repo_error)?;
        Ok(rows_affected)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
