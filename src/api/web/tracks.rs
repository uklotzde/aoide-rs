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

use aoide_storage::{
    api::{
        serde::{SerializationFormat, SerializedEntity},
        track::{TrackAlbums, TrackTags, Tracks, TracksResult},
        CountTagAvgScoresParams, CountTagFacetsParams, CountTrackAlbumsParams, LocateTracksParams,
        Pagination, ReplaceTracksParams, ReplacedTracks, SearchTracksParams, TagAvgScoreCount,
        TagFacetCount,
    },
    storage::track::TrackRepository,
};

use actix_web::AsyncResponder;

use futures::future::Future;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TracksQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_uid: Option<EntityUid>,
}

#[derive(Debug)]
pub struct CreateTrackMessage {
    pub track: Track,
}

pub type CreateTrackResult = TracksResult<TrackEntity>;

impl Message for CreateTrackMessage {
    type Result = CreateTrackResult;
}

impl Handler<CreateTrackMessage> for SqliteExecutor {
    type Result = CreateTrackResult;

    fn handle(&mut self, msg: CreateTrackMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.create_entity(msg.track, SerializationFormat::JSON)
        })
    }
}

pub fn on_create_track(
    (state, body): (State<AppState>, Json<Track>),
) -> FutureResponse<HttpResponse> {
    let msg = CreateTrackMessage {
        track: body.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Created().json(res.header())))
        .responder()
}

#[derive(Debug)]
pub struct UpdateTrackMessage {
    pub track: TrackEntity,
}

pub type UpdateTrackResult = TracksResult<(EntityRevision, Option<EntityRevision>)>;

impl Message for UpdateTrackMessage {
    type Result = UpdateTrackResult;
}

impl Handler<UpdateTrackMessage> for SqliteExecutor {
    type Result = UpdateTrackResult;

    fn handle(&mut self, msg: UpdateTrackMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.update_entity(msg.track, SerializationFormat::JSON)
        })
    }
}

pub fn on_update_track(
    (state, path_uid, body): (State<AppState>, Path<EntityUid>, Json<TrackEntity>),
) -> FutureResponse<HttpResponse> {
    let uid = path_uid.into_inner();
    let msg = UpdateTrackMessage {
        track: body.into_inner(),
    };
    // TODO: Handle UID mismatch
    assert!(uid == *msg.track.header().uid());
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(move |res| match res {
            (_, Some(next_revision)) => {
                let next_header = EntityHeader::new(uid, next_revision);
                Ok(HttpResponse::Ok().json(next_header))
            }
            (_, None) => Err(actix_web::error::ErrorBadRequest(failure::format_err!(
                "Inexistent entity or revision conflict"
            ))),
        })
        .responder()
}

#[derive(Debug)]
pub struct DeleteTrackMessage {
    pub uid: EntityUid,
}

pub type DeleteTrackResult = TracksResult<Option<()>>;

impl Message for DeleteTrackMessage {
    type Result = DeleteTrackResult;
}

impl Handler<DeleteTrackMessage> for SqliteExecutor {
    type Result = DeleteTrackResult;

    fn handle(&mut self, msg: DeleteTrackMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| repository.delete_entity(&msg.uid))
    }
}

pub fn on_delete_track(
    (state, path_uid): (State<AppState>, Path<EntityUid>),
) -> FutureResponse<HttpResponse> {
    let msg = DeleteTrackMessage {
        uid: path_uid.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(|res| match res {
            Some(_) => Ok(HttpResponse::NoContent().into()),
            None => Ok(HttpResponse::NotFound().into()),
        })
        .responder()
}

#[derive(Debug)]
pub struct LoadTrackMessage {
    pub uid: EntityUid,
}

pub type LoadTrackResult = TracksResult<Option<SerializedEntity>>;

impl Message for LoadTrackMessage {
    type Result = LoadTrackResult;
}

impl Handler<LoadTrackMessage> for SqliteExecutor {
    type Result = LoadTrackResult;

    fn handle(&mut self, msg: LoadTrackMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| repository.load_entity(&msg.uid))
    }
}

pub fn on_load_track(
    (state, path_uid): (State<AppState>, Path<EntityUid>),
) -> FutureResponse<HttpResponse> {
    let msg = LoadTrackMessage {
        uid: path_uid.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(|res| match res {
            Some(serialized_track) => {
                let mime_type: mime::Mime = serialized_track.format.into();
                Ok(HttpResponse::Ok()
                    .content_type(mime_type.to_string().as_str())
                    .body(serialized_track.blob))
            }
            None => Ok(HttpResponse::NotFound().into()),
        })
        .responder()
}

#[derive(Debug, Default)]
pub struct SearchTracksMessage {
    pub collection_uid: Option<EntityUid>,
    pub pagination: Pagination,
    pub params: SearchTracksParams,
}

pub type SearchTracksResult = TracksResult<Vec<SerializedEntity>>;

impl Message for SearchTracksMessage {
    type Result = SearchTracksResult;
}

impl Handler<SearchTracksMessage> for SqliteExecutor {
    type Result = SearchTracksResult;

    fn handle(&mut self, msg: SearchTracksMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.search_entities(msg.collection_uid.as_ref(), msg.pagination, msg.params)
        })
    }
}

pub fn on_list_tracks(
    (state, query_tracks, query_pagination): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<Pagination>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = SearchTracksMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        pagination: query_pagination.into_inner(),
        ..Default::default()
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(|serialized_tracks| SerializedEntity::slice_to_json_array(&serialized_tracks))
        .from_err()
        .and_then(|json| {
            Ok(HttpResponse::Ok()
                .content_type(mime::APPLICATION_JSON.to_string().as_str())
                .body(json))
        })
        .responder()
}

pub fn on_search_tracks(
    (state, query_tracks, query_pagination, body): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<Pagination>,
        Json<SearchTracksParams>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = SearchTracksMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        params: body.into_inner(),
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(|serialized_tracks| SerializedEntity::slice_to_json_array(&serialized_tracks))
        .from_err()
        .and_then(|json| {
            Ok(HttpResponse::Ok()
                .content_type(mime::APPLICATION_JSON.to_string().as_str())
                .body(json))
        })
        .responder()
}

#[derive(Debug)]
pub struct LocateTracksMessage {
    pub collection_uid: Option<EntityUid>,
    pub pagination: Pagination,
    pub params: LocateTracksParams,
}

pub type LocateTracksResult = TracksResult<Vec<SerializedEntity>>;

impl Message for LocateTracksMessage {
    type Result = LocateTracksResult;
}

impl Handler<LocateTracksMessage> for SqliteExecutor {
    type Result = LocateTracksResult;

    fn handle(&mut self, msg: LocateTracksMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.locate_entities(msg.collection_uid.as_ref(), msg.pagination, msg.params)
        })
    }
}

pub fn on_locate_tracks(
    (state, query_tracks, query_pagination, body): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<Pagination>,
        Json<LocateTracksParams>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = LocateTracksMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        params: body.into_inner(),
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(|serialized_tracks| SerializedEntity::slice_to_json_array(&serialized_tracks))
        .from_err()
        .and_then(|json| {
            Ok(HttpResponse::Ok()
                .content_type(mime::APPLICATION_JSON.to_string().as_str())
                .body(json))
        })
        .responder()
}

#[derive(Debug)]
pub struct ReplaceTracksMessage {
    pub collection_uid: Option<EntityUid>,
    pub params: ReplaceTracksParams,
    pub format: SerializationFormat,
}

pub type ReplaceTracksResult = TracksResult<ReplacedTracks>;

impl Message for ReplaceTracksMessage {
    type Result = ReplaceTracksResult;
}

impl Handler<ReplaceTracksMessage> for SqliteExecutor {
    type Result = ReplaceTracksResult;

    fn handle(&mut self, msg: ReplaceTracksMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.replace_entities(msg.collection_uid.as_ref(), msg.params, msg.format)
        })
    }
}

pub fn on_replace_tracks(
    (state, query_tracks, body): (
        State<AppState>,
        Query<TracksQueryParams>,
        Json<ReplaceTracksParams>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = ReplaceTracksMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        params: body.into_inner(),
        format: SerializationFormat::JSON,
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}

#[derive(Debug, Default)]
struct CountTrackAlbumsMessage {
    pub collection_uid: Option<EntityUid>,
    pub params: CountTrackAlbumsParams,
    pub pagination: Pagination,
}

pub type CountTrackAlbumsResult = TracksResult<Vec<TrackAlbumCount>>;

impl Message for CountTrackAlbumsMessage {
    type Result = CountTrackAlbumsResult;
}

impl Handler<CountTrackAlbumsMessage> for SqliteExecutor {
    type Result = CountTrackAlbumsResult;

    fn handle(&mut self, msg: CountTrackAlbumsMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.count_albums(msg.collection_uid.as_ref(), &msg.params, msg.pagination)
        })
    }
}

pub fn on_count_track_albums(
    (state, query_tracks, query_pagination, body): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<Pagination>,
        Json<CountTrackAlbumsParams>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = CountTrackAlbumsMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        params: body.into_inner(),
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}

#[derive(Debug, Default)]
struct CountTrackTagAvgScoresMessage {
    pub collection_uid: Option<EntityUid>,
    pub pagination: Pagination,
    pub params: CountTagAvgScoresParams,
}

pub type CountTrackTagAvgScoresResult = TracksResult<Vec<TagAvgScoreCount>>;

impl Message for CountTrackTagAvgScoresMessage {
    type Result = CountTrackTagAvgScoresResult;
}

impl Handler<CountTrackTagAvgScoresMessage> for SqliteExecutor {
    type Result = CountTrackTagAvgScoresResult;

    fn handle(
        &mut self,
        msg: CountTrackTagAvgScoresMessage,
        _: &mut Self::Context,
    ) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        let collection_uid = msg.collection_uid;
        let pagination = msg.pagination;
        let include_non_faceted_tags = msg.params.include_non_faceted_tags;
        let facets = msg.params.facets.map(|mut facets| {
            facets.sort();
            facets.dedup();
            facets
        });
        let facets = facets.as_ref().map(|facets| {
            facets
                .iter()
                .map(AsRef::as_ref)
                .map(String::as_str)
                .collect()
        });
        connection.transaction::<_, Error, _>(|| {
            repository.count_tag_avg_scores(
                collection_uid.as_ref(),
                facets.as_ref().map(Vec::as_slice),
                include_non_faceted_tags,
                pagination,
            )
        })
    }
}

pub fn on_count_tag_avg_scores(
    (state, query_tracks, query_pagination, body): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<Pagination>,
        Json<CountTagAvgScoresParams>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = CountTrackTagAvgScoresMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        params: body.into_inner(),
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}

#[derive(Debug, Default)]
struct CountTrackTagFacetsMessage {
    pub collection_uid: Option<EntityUid>,
    pub pagination: Pagination,
    pub params: CountTagFacetsParams,
}

pub type CountTrackTagFacetsResult = TracksResult<Vec<TagFacetCount>>;

impl Message for CountTrackTagFacetsMessage {
    type Result = CountTrackTagFacetsResult;
}

impl Handler<CountTrackTagFacetsMessage> for SqliteExecutor {
    type Result = CountTrackTagFacetsResult;

    fn handle(&mut self, msg: CountTrackTagFacetsMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        let collection_uid = msg.collection_uid;
        let pagination = msg.pagination;
        let facets = msg.params.facets.map(|mut facets| {
            facets.sort();
            facets.dedup();
            facets
        });
        let facets = facets.as_ref().map(|facets| {
            facets
                .iter()
                .map(AsRef::as_ref)
                .map(String::as_str)
                .collect()
        });
        connection.transaction::<_, Error, _>(|| {
            repository.count_tag_facets(
                collection_uid.as_ref(),
                facets.as_ref().map(Vec::as_slice),
                pagination,
            )
        })
    }
}

pub fn on_count_track_facets(
    (state, query_tracks, query_pagination, body): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<Pagination>,
        Json<CountTagFacetsParams>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = CountTrackTagFacetsMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        params: body.into_inner(),
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}
