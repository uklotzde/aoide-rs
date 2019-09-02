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

use crate::usecases::collections::*;

mod _core {
    pub use aoide_core::{
        collection::{Collection, Entity},
        entity::{EntityHeader, EntityRevision, EntityUid},
    };
}

mod _repo {
    pub use aoide_repo::collection::TrackStats;
}

use aoide_core::util::IsDefault;

use aoide_core_serde::{
    collection::{Collection, Entity},
    entity::EntityHeader,
};

///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct TrackStats {
    pub total_count: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct EntityStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracks: Option<TrackStats>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct EntityWithStats {
    pub entity: Entity,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub stats: EntityStats,
}

pub struct CollectionsHandler {
    db: SqlitePooledConnection,
}

impl CollectionsHandler {
    pub fn new(db: SqlitePooledConnection) -> Self {
        Self { db }
    }

    pub fn handle_create(
        &self,
        new_collection: Collection,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        create_collection(&self.db, new_collection.into())
            .map_err(warp::reject::custom)
            .map(|hdr| {
                warp::reply::with_status(
                    warp::reply::json(&EntityHeader::from(hdr)),
                    warp::http::StatusCode::CREATED,
                )
            })
    }

    pub fn handle_update(
        &self,
        uid: _core::EntityUid,
        entity: Entity,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        let entity = _core::Entity::from(entity);
        if uid != entity.hdr.uid {
            return Err(warp::reject::custom(failure::format_err!(
                "Mismatching UIDs: {} <> {}",
                uid,
                entity.hdr.uid,
            )));
        }
        update_collection(&self.db, &entity)
            .and_then(move |res| match res {
                (_, Some(rev)) => {
                    let hdr = _core::EntityHeader { uid, rev };
                    Ok(warp::reply::json(&EntityHeader::from(hdr)))
                }
                (_, None) => Err(failure::format_err!(
                    "Inexistent entity or revision conflict"
                )),
            })
            .map_err(warp::reject::custom)
    }

    pub fn handle_delete(
        &self,
        uid: _core::EntityUid,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        delete_collection(&self.db, &uid)
            .map_err(warp::reject::custom)
            .map(|res| {
                warp::reply::with_status(
                    warp::reply(),
                    res.map(|()| warp::http::StatusCode::NO_CONTENT)
                        .unwrap_or(warp::http::StatusCode::NOT_FOUND),
                )
            })
    }

    pub fn handle_load(
        &self,
        uid: _core::EntityUid,
        params: WithTokensQueryParams,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        load_collection(&self.db, &uid, params.try_with_token("track-stats"))
            .map_err(warp::reject::custom)
            .and_then(|res| match res {
                Some((entity, track_stats)) => {
                    let stats = EntityStats {
                        tracks: track_stats.map(|track_stats| TrackStats {
                            total_count: track_stats.total_count,
                        }),
                    };
                    let entity_with_stats = EntityWithStats {
                        entity: entity.into(),
                        stats,
                    };
                    Ok(warp::reply::json(&entity_with_stats))
                }
                None => Err(warp::reject::not_found()),
            })
    }

    pub fn handle_list(
        &self,
        pagination: PaginationQueryParams,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        list_collections(&self.db, pagination.into())
            .map_err(warp::reject::custom)
            .map(|entities| {
                let entities: Vec<_> = entities.into_iter().map(Entity::from).collect();
                warp::reply::json(&entities)
            })
    }
}
