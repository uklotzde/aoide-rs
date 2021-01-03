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

#![warn(rust_2018_idioms)]

mod env;

use aoide::{
    api::web::{collections, playlists, reject_from_anyhow, tracks},
    usecases as uc, *,
};

use aoide_core::entity::EntityUid;

mod _serde {
    pub use aoide_core_serde::entity::EntityUid;
}

use aoide_repo::prelude::RepoError;

use anyhow::Error;
use futures::future::{join, FutureExt};
use std::{env::current_exe, time::Duration};
use tokio::{sync::mpsc, time::delay_for};
use warp::{http::StatusCode, Filter};

///////////////////////////////////////////////////////////////////////

const WEB_SERVER_LISTENING_DELAY: Duration = Duration::from_millis(250);

static INDEX_HTML: &str = include_str!("../../../resources/index.html");
static OPENAPI_YAML: &str = include_str!("../../../resources/openapi.yaml");

fn create_connection_pool(
    database_url: &str,
    max_size: u32,
) -> Result<SqliteConnectionPool, Error> {
    log::info!("Creating SQLite connection pool");
    let manager = SqliteConnectionManager::new(database_url);
    let pool = SqliteConnectionPool::builder()
        .max_size(max_size)
        .build(manager)?;
    Ok(pool)
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let started_at = chrono::Utc::now();

    env::init_environment();

    env::init_logging();

    if let Ok(exe_path) = current_exe() {
        log::info!("Executable: {}", exe_path.display());
    }
    log::info!("Version: {}", env!("CARGO_PKG_VERSION"));

    let endpoint_addr = env::parse_endpoint_addr();
    log::info!("Endpoint address: {}", endpoint_addr);

    let database_url = env::parse_database_url();
    log::info!("Database URL: {}", database_url);

    // Workaround: Use a pool of size 1 to avoid 'database is locked'
    // errors due to multi-threading.
    let connection_pool = create_connection_pool(&database_url, 1)
        .expect("Failed to create database connection pool");

    uc::database::initialize(&*connection_pool.get()?).expect("Failed to initialize database");
    uc::database::migrate_schema(&*connection_pool.get()?)
        .expect("Failed to migrate database schema");

    let sqlite_exec = SqliteExecutor::new(connection_pool.clone());

    log::info!("Creating service routes");

    let pooled_connection = warp::any()
        .map(move || sqlite_exec.pooled_connection())
        .and_then(|res: Result<_, _>| async { res.map_err(reject_from_anyhow) });

    // POST /shutdown
    let (server_shutdown_tx, mut server_shutdown_rx) = mpsc::unbounded_channel::<()>();
    let shutdown_filter = warp::post()
        .and(warp::path("shutdown"))
        .and(warp::path::end())
        .map(move || {
            server_shutdown_tx
                .send(())
                .map(|()| StatusCode::ACCEPTED)
                .or_else(|_| {
                    log::warn!("Failed to forward shutdown request");
                    Ok(StatusCode::BAD_GATEWAY)
                })
        });

    // GET /about
    let about_filter = warp::get()
        .and(warp::path("about"))
        .and(warp::path::end())
        .map(move || {
            warp::reply::json(&serde_json::json!({
            "name": env!("CARGO_PKG_NAME"),
            "description": env!("CARGO_PKG_DESCRIPTION"),
            "version": env!("CARGO_PKG_VERSION"),
            "authors": env!("CARGO_PKG_AUTHORS"),
            "instance": {
                "startedAt": started_at,
                }
            }))
        });

    let path_param_uid = warp::path::param::<EntityUid>();

    let collections_path = warp::path("c");
    let tracks_path = warp::path("tracks");
    let playlists_path = warp::path("playlists");
    let storage_path = warp::path("storage");

    // Collections
    let collections_create = warp::post()
        .and(collections_path)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|request_body, pooled_connection| async move {
            collections::create::handle_request(&pooled_connection, request_body)
                .map(|response_body| {
                    warp::reply::with_status(warp::reply::json(&response_body), StatusCode::CREATED)
                })
                .map_err(anyhow::Error::from)
                .map_err(reject_from_anyhow)
        });
    let collections_update = warp::put()
        .and(collections_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(
            |uid, query_params, request_body, pooled_connection| async move {
                collections::update::handle_request(
                    &pooled_connection,
                    uid,
                    query_params,
                    request_body,
                )
                .map(|response_body| {
                    warp::reply::with_status(warp::reply::json(&response_body), StatusCode::OK)
                })
                .or_else(|err| match err {
                    RepoError::NotFound => Ok(warp::reply::with_status(
                        warp::reply::json(&()),
                        StatusCode::NOT_FOUND,
                    )),
                    RepoError::Conflict => Ok(warp::reply::with_status(
                        warp::reply::json(&()),
                        StatusCode::CONFLICT,
                    )),
                    err => Err(reject_from_anyhow(err.into())),
                })
            },
        );
    let collections_delete = warp::delete()
        .and(collections_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| async move {
            collections::delete::handle_request(&pooled_connection, &uid)
                .map(|()| StatusCode::NO_CONTENT)
                .or_else(|err| match err {
                    RepoError::NotFound => Ok(StatusCode::NOT_FOUND),
                    err => Err(err),
                })
                .map_err(anyhow::Error::from)
                .map_err(reject_from_anyhow)
        });
    let collections_list = warp::get()
        .and(collections_path)
        .and(warp::path::end())
        .and(warp::query())
        .and(pooled_connection.clone())
        .and_then(|query_params, pooled_connection| async move {
            collections::load_all::handle_request(&pooled_connection, query_params)
                .map(|response_body| warp::reply::json(&response_body))
                .map_err(anyhow::Error::from)
                .map_err(reject_from_anyhow)
        });
    let collections_get = warp::get()
        .and(collections_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(warp::query())
        .and(pooled_connection.clone())
        .and_then(|uid, query_params, pooled_connection| async move {
            collections::load_one::handle_request(&pooled_connection, &uid, query_params)
                .map(|response_body| {
                    warp::reply::with_status(warp::reply::json(&response_body), StatusCode::OK)
                })
                .or_else(|err| match err {
                    RepoError::NotFound => Ok(warp::reply::with_status(
                        warp::reply::json(&()),
                        StatusCode::NOT_FOUND,
                    )),
                    err => Err(reject_from_anyhow(err.into())),
                })
        });
    let collections_filters = collections_list
        .or(collections_get)
        .or(collections_create)
        .or(collections_update)
        .or(collections_delete);

    let collected_tracks_resolve = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(tracks_path)
        .and(warp::path("resolve"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|uid, request_body, pooled_connection| async move {
            tracks::resolve_collected::handle_request(&pooled_connection, &uid, request_body)
                .map(|response_body| warp::reply::json(&response_body))
                .map_err(anyhow::Error::from)
                .map_err(reject_from_anyhow)
        });
    let collected_tracks_search = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(tracks_path)
        .and(warp::path("search"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(
            |uid, query_params, request_body, pooled_connection| async move {
                tracks::search_collected::handle_request(
                    &pooled_connection,
                    &uid,
                    query_params,
                    request_body,
                )
                .map(|response_body| warp::reply::json(&response_body))
                .map_err(anyhow::Error::from)
                .map_err(reject_from_anyhow)
            },
        );
    let collected_tracks_replace = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(tracks_path)
        .and(warp::path("replace"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(
            |uid, query_params, request_body, pooled_connection| async move {
                tracks::replace_collected::handle_request(
                    &pooled_connection,
                    &uid,
                    query_params,
                    request_body,
                )
                .map(|response_body| warp::reply::json(&response_body))
                .map_err(anyhow::Error::from)
                .map_err(reject_from_anyhow)
            },
        );
    let collected_tracks_purge = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(tracks_path)
        .and(warp::path("purge"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|uid, request_body, pooled_connection| async move {
            tracks::purge_collected::handle_request(&pooled_connection, &uid, request_body)
                .map(|response_body| warp::reply::json(&response_body))
                .map_err(anyhow::Error::from)
                .map_err(reject_from_anyhow)
        });
    let collected_tracks_filters = collected_tracks_resolve
        .or(collected_tracks_search)
        .or(collected_tracks_replace)
        .or(collected_tracks_purge);

    // Tracks
    let tracks_load_one = warp::get()
        .and(tracks_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| async move {
            tracks::load_one::handle_request(&pooled_connection, &uid)
                .map(|response_body| {
                    warp::reply::with_status(warp::reply::json(&response_body), StatusCode::OK)
                })
                .or_else(|err| match err {
                    RepoError::NotFound => Ok(warp::reply::with_status(
                        warp::reply::json(&()),
                        StatusCode::NOT_FOUND,
                    )),
                    err => Err(reject_from_anyhow(err.into())),
                })
        });
    let tracks_load_many = warp::post()
        .and(tracks_path)
        .and(warp::path("load"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|request_body, pooled_connection| async move {
            tracks::load_many::handle_request(&pooled_connection, request_body)
                .map(|response_body| warp::reply::json(&response_body))
                .map_err(anyhow::Error::from)
                .map_err(reject_from_anyhow)
        });
    let tracks_filters = tracks_load_many.or(tracks_load_one);

    let collected_playlists_create = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(playlists_path)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(
            |collection_uid, request_body, pooled_connection| async move {
                playlists::create_collected::handle_request(
                    &pooled_connection,
                    &collection_uid,
                    request_body,
                )
                .map(|response_body| {
                    warp::reply::with_status(warp::reply::json(&response_body), StatusCode::CREATED)
                })
                .map_err(anyhow::Error::from)
                .map_err(reject_from_anyhow)
            },
        );
    let collected_playlists_list = warp::get()
        .and(collections_path)
        .and(path_param_uid)
        .and(playlists_path)
        .and(warp::path::end())
        .and(warp::query())
        .and(pooled_connection.clone())
        .and_then(
            |collection_uid, query_params, pooled_connection| async move {
                playlists::list_collected::handle_request(
                    &pooled_connection,
                    &collection_uid,
                    query_params,
                )
                .map(|response_body| warp::reply::json(&response_body))
                .map_err(anyhow::Error::from)
                .map_err(reject_from_anyhow)
            },
        );
    let collected_playlists_filters = collected_playlists_list.or(collected_playlists_create);

    let playlists_update = warp::put()
        .and(playlists_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(
            |uid, query_params, request_body, pooled_connection| async move {
                playlists::update::handle_request(
                    &pooled_connection,
                    uid,
                    query_params,
                    request_body,
                )
                .map(|response_body| {
                    warp::reply::with_status(warp::reply::json(&response_body), StatusCode::OK)
                })
                .or_else(|err| match err {
                    RepoError::NotFound => Ok(warp::reply::with_status(
                        warp::reply::json(&()),
                        StatusCode::NOT_FOUND,
                    )),
                    RepoError::Conflict => Ok(warp::reply::with_status(
                        warp::reply::json(&()),
                        StatusCode::CONFLICT,
                    )),
                    err => Err(reject_from_anyhow(err.into())),
                })
            },
        );
    let playlists_delete = warp::delete()
        .and(playlists_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| async move {
            playlists::delete::handle_request(&pooled_connection, &uid)
                .map(|()| StatusCode::NO_CONTENT)
                .or_else(|err| match err {
                    RepoError::NotFound => Ok(StatusCode::NOT_FOUND),
                    err => Err(err),
                })
                .map_err(anyhow::Error::from)
                .map_err(reject_from_anyhow)
        });
    let playlists_entries_patch = warp::patch()
        .and(playlists_path)
        .and(path_param_uid)
        .and(warp::path("entries"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(
            |uid, query_params, request_body, pooled_connection| async move {
                playlists::patch_entries::handle_request(
                    &pooled_connection,
                    uid,
                    query_params,
                    request_body,
                )
                .map(|response_body| {
                    warp::reply::with_status(warp::reply::json(&response_body), StatusCode::OK)
                })
                .or_else(|err| match err {
                    RepoError::NotFound => Ok(warp::reply::with_status(
                        warp::reply::json(&()),
                        StatusCode::NOT_FOUND,
                    )),
                    RepoError::Conflict => Ok(warp::reply::with_status(
                        warp::reply::json(&()),
                        StatusCode::CONFLICT,
                    )),
                    err => Err(err),
                })
                .map_err(anyhow::Error::from)
                .map_err(reject_from_anyhow)
            },
        );
    let playlists_filters = playlists_update
        .or(playlists_delete)
        .or(playlists_entries_patch);

    // Storage
    let storage_groom = warp::post()
        .and(storage_path)
        .and(warp::path("groom"))
        .and(warp::path::end())
        .and(pooled_connection.clone())
        .and_then(|pooled_connection: SqlitePooledConnection| async move {
            uc::database::groom(&*pooled_connection)
                .map(|()| StatusCode::NO_CONTENT)
                .map_err(Into::into)
                .map_err(reject_from_anyhow)
        });
    let storage_optimize = warp::post()
        .and(storage_path)
        .and(warp::path("optimize"))
        .and(warp::path::end())
        .and(pooled_connection.clone())
        .and_then(|pooled_connection: SqlitePooledConnection| async move {
            uc::database::optimize(&*pooled_connection)
                .map(|()| StatusCode::NO_CONTENT)
                .map_err(reject_from_anyhow)
        });
    let storage_filters = storage_groom.or(storage_optimize);

    // Static content
    let index_html = warp::path::end().map(|| warp::reply::html(INDEX_HTML));
    let openapi_yaml = warp::path("openapi.yaml").map(|| {
        warp::reply::with_header(
            OPENAPI_YAML,
            "Content-Type",
            "application/x-yaml;charset=utf-8",
        )
    });
    let static_filters = index_html.or(openapi_yaml);

    log::info!("Initializing server");

    let cors = warp::cors().allow_any_origin();
    let server = warp::serve(
        collections_filters
            .or(collected_tracks_filters)
            .or(tracks_filters)
            .or(collected_playlists_filters)
            .or(playlists_filters)
            .or(storage_filters)
            .or(static_filters)
            .or(shutdown_filter)
            .or(about_filter)
            .with(cors),
    );

    log::info!("Starting");

    let (socket_addr, server_listener) =
        server.bind_with_graceful_shutdown(endpoint_addr, async move {
            server_shutdown_rx.recv().await;
            log::info!("Stopping");
        });

    let server_listening = async move {
        // Give the server some time to become ready and start listening
        // before announcing the actual endpoint address, i.e. when using
        // an ephemeral port. The delay might need to be tuned depending
        // on how long the startup actually takes. Unfortunately warp does
        // not provide any signal when the server has started listening.
        delay_for(WEB_SERVER_LISTENING_DELAY).await;

        // -> stderr
        log::info!("Listening on {}", socket_addr);
        // -> stdout
        println!("{}", socket_addr);
    };

    join(server_listener, server_listening).map(drop).await;
    log::info!("Stopped");

    Ok(())
}
