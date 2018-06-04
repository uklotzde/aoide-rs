// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

pub mod api;

use failure;

use self::api::{Pagination, LocateParams, TrackReplacementParams, TrackReplacementReport, SearchParams, CountableStringField, StringFieldCounts, ResourceStats};

use storage::serde::{SerializationFormat, SerializedEntity};

use aoide_core::domain::entity::*;
use aoide_core::domain::collection::*;
use aoide_core::domain::metadata::*;
use aoide_core::domain::track::*;

pub type CollectionsResult<T> = Result<T, failure::Error>;

pub trait Collections {
    fn create_entity(&self, body: CollectionBody) -> CollectionsResult<CollectionEntity>;

    fn insert_entity(&self, entity: &CollectionEntity) -> CollectionsResult<()>;

    fn update_entity(&self, entity: &CollectionEntity)
        -> CollectionsResult<Option<(EntityRevision, EntityRevision)>>;

    fn remove_entity(&self, uid: &EntityUid) -> CollectionsResult<Option<()>>;

    fn find_entity(&self, uid: &EntityUid) -> CollectionsResult<Option<CollectionEntity>>;

    fn find_recently_revisioned_entities(
        &self,
        pagination: &Pagination,
    ) -> CollectionsResult<Vec<CollectionEntity>>;

    fn find_entities_by_name(&self, name: &str) -> CollectionsResult<Vec<CollectionEntity>>;

    fn find_entities_by_name_starting_with(
        &self,
        name: &str,
        pagination: &Pagination,
    ) -> CollectionsResult<Vec<CollectionEntity>>;

    fn find_entities_by_name_containing(
        &self,
        name: &str,
        pagination: &Pagination,
    ) -> CollectionsResult<Vec<CollectionEntity>>;
}

pub type TracksResult<T> = Result<T, failure::Error>;

pub trait Tracks {
    fn create_entity(
        &self,
        body: TrackBody,
        format: SerializationFormat,
    ) -> TracksResult<TrackEntity>;

    fn insert_entity(
        &self,
        entity: &TrackEntity,
        format: SerializationFormat,
    ) -> TracksResult<()>;

    fn update_entity(
        &self,
        entity: &mut TrackEntity,
        format: SerializationFormat,
    ) -> TracksResult<Option<(EntityRevision, EntityRevision)>>;

    fn replace_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        replace_params: TrackReplacementParams,
        format: SerializationFormat,
    ) -> TracksResult<TrackReplacementReport>;

    fn remove_entity(&self, uid: &EntityUid) -> TracksResult<()>;

    fn load_entity(&self, uid: &EntityUid) -> TracksResult<Option<SerializedEntity>>;

    fn locate_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: &Pagination,
        locate_params: LocateParams
    ) -> TracksResult<Vec<SerializedEntity>>;

    fn search_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: &Pagination,
        search_params: SearchParams,
    ) -> TracksResult<Vec<SerializedEntity>>;

    fn field_counts(
        &self,
        collection_uid: Option<&EntityUid>,
        field: CountableStringField,
    ) -> TracksResult<StringFieldCounts>;

    fn resource_statistics(
        &self,
        collection_uid: Option<&EntityUid>,
    ) -> TracksResult<ResourceStats>;
}

pub type TrackTaggingsResult<T> = Result<T, failure::Error>;

pub trait TrackTaggings {
    fn all_tags_facets(
        &self,
        collection_uid: Option<&EntityUid>,
        facets: Option<&Vec<&str>>,
        pagination: &Pagination,
    ) -> TrackTaggingsResult<Vec<TagFacetCount>>;

    fn all_tags(
        &self,
        collection_uid: Option<&EntityUid>,
        facets: Option<&Vec<&str>>,
        pagination: &Pagination,
    ) -> TrackTaggingsResult<Vec<ScoredTagCount>>;
}