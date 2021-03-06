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

mod uc {
    pub use crate::usecases::tracks::replace::*;

    pub use aoide_repo::track::ReplaceMode;

    pub use aoide_usecases::tracks::replace::{Params, Summary};
}

pub use aoide_core_serde::{
    entity::EntityHeader,
    track::{Entity, Track},
};

///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum ReplaceMode {
    CreateOnly,
    UpdateOnly,
    UpdateOrCreate,
}

impl From<ReplaceMode> for uc::ReplaceMode {
    fn from(from: ReplaceMode) -> Self {
        use ReplaceMode::*;
        match from {
            CreateOnly => Self::CreateOnly,
            UpdateOnly => Self::UpdateOnly,
            UpdateOrCreate => Self::UpdateOrCreate,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Summary {
    pub created: Vec<Entity>,
    pub updated: Vec<Entity>,
    pub unchanged: Vec<String>,
    pub not_created: Vec<Track>,
    pub not_updated: Vec<Track>,
}

impl From<uc::Summary> for Summary {
    fn from(from: uc::Summary) -> Self {
        let uc::Summary {
            created,
            updated,
            unchanged,
            not_imported: _not_imported,
            not_created,
            not_updated,
        } = from;
        debug_assert!(_not_imported.is_empty());
        Self {
            created: created.into_iter().map(Into::into).collect(),
            updated: updated.into_iter().map(Into::into).collect(),
            unchanged: unchanged.into_iter().map(Into::into).collect(),
            not_created: not_created.into_iter().map(Into::into).collect(),
            not_updated: not_updated.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ReplaceMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_path_from_url: Option<bool>,
}

pub type RequestBody = Vec<Track>;

pub type ResponseBody = Summary;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &_core::EntityUid,
    query_params: QueryParams,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let QueryParams {
        mode,
        resolve_path_from_url,
    } = query_params;
    let replace_mode = mode.unwrap_or(ReplaceMode::UpdateOrCreate);
    let resolve_path_from_url = resolve_path_from_url.unwrap_or(false);
    let params = uc::Params {
        resolve_path_from_url,
        ..uc::Params::new(replace_mode.into())
    };
    Ok(uc::replace_by_media_source_path(
        &pooled_connection,
        collection_uid,
        &params,
        request_body.into_iter().map(Into::into),
    )
    .map(Into::into)?)
}
