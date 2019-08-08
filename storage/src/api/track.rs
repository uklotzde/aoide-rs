// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::serde::{SerializationFormat, SerializedEntity};

use crate::api::{
    collection::CollectionTrackStats, CountTracksByAlbumParams, FieldStrings, LocateTracksParams,
    Pagination, ReplaceTracksParams, ReplacedTracks, SearchTracksParams, StringField, TagCount,
    TagFacetCount,
};

use failure::Error;

///////////////////////////////////////////////////////////////////////

pub type TracksResult<T> = Result<T, Error>;

pub trait Tracks {
    fn create_entity(&self, body: Track, format: SerializationFormat) -> TracksResult<TrackEntity>;

    fn insert_entity(&self, entity: &TrackEntity, format: SerializationFormat) -> TracksResult<()>;

    fn update_entity(
        &self,
        entity: TrackEntity,
        format: SerializationFormat,
    ) -> TracksResult<(EntityRevision, Option<EntityRevision>)>;

    fn replace_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        replace_params: ReplaceTracksParams,
        format: SerializationFormat,
    ) -> TracksResult<ReplacedTracks>;

    fn delete_entity(&self, uid: &EntityUid) -> TracksResult<Option<()>>;

    fn load_entity(&self, uid: &EntityUid) -> TracksResult<Option<SerializedEntity>>;

    fn locate_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: Pagination,
        locate_params: LocateTracksParams,
    ) -> TracksResult<Vec<SerializedEntity>>;

    fn search_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: Pagination,
        search_params: SearchTracksParams,
    ) -> TracksResult<Vec<SerializedEntity>>;

    fn list_field_strings(
        &self,
        collection_uid: Option<&EntityUid>,
        field: StringField,
        pagination: Pagination,
    ) -> TracksResult<FieldStrings>;

    fn collection_stats(&self, collection_uid: &EntityUid) -> TracksResult<CollectionTrackStats>;
}

pub type TrackAlbumsResult<T> = Result<T, Error>;

pub trait TrackAlbums {
    fn count_tracks_by_album(
        &self,
        collection_uid: Option<&EntityUid>,
        params: &CountTracksByAlbumParams,
        pagination: Pagination,
    ) -> TracksResult<Vec<AlbumTracksCount>>;
}

pub type TrackTagsResult<T> = Result<T, Error>;

pub trait TrackTags {
    fn count_tracks_by_tag_facet(
        &self,
        collection_uid: Option<&EntityUid>,
        params: &CountTracksByTagFacetParams,
        pagination: Pagination,
    ) -> TrackTagsResult<Vec<TagFacetCount>>;

    fn count_tracks_by_tag(
        &self,
        collection_uid: Option<&EntityUid>,
        params: &CountTracksByTagParams,
        pagination: Pagination,
    ) -> TrackTagsResult<Vec<TagCount>>;
}
