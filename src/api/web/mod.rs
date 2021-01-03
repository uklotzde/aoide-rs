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

use serde::{Deserialize, Serialize};

pub mod collections;
pub mod playlists;
pub mod tracks;

mod json;

use warp::reject::{self, Reject, Rejection};

use std::{error::Error as StdError, fmt};

#[derive(Debug)]
struct RejectAnyhowError(anyhow::Error);

impl fmt::Display for RejectAnyhowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Reject for RejectAnyhowError {}

impl StdError for RejectAnyhowError {}

impl From<anyhow::Error> for RejectAnyhowError {
    fn from(err: anyhow::Error) -> Self {
        RejectAnyhowError(err)
    }
}

pub fn reject_from_anyhow(err: anyhow::Error) -> Rejection {
    reject::custom(RejectAnyhowError(err))
}

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaginationQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<aoide_repo::PaginationOffset>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<aoide_repo::PaginationLimit>,
}

impl From<PaginationQueryParams> for aoide_repo::Pagination {
    fn from(from: PaginationQueryParams) -> Self {
        let PaginationQueryParams { offset, limit } = from;
        Self { offset, limit }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WithTokensQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    with: Option<String>,
}

impl WithTokensQueryParams {
    pub fn try_with_token(&self, with_token: &str) -> bool {
        match self.with {
            Some(ref with) => with.split(',').any(|token| token == with_token),
            None => false,
        }
    }
}
