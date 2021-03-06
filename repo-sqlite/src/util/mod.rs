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

pub mod clock;
pub mod entity;

use crate::prelude::*;

use num_traits::ToPrimitive as _;
use std::i64;

///////////////////////////////////////////////////////////////////////

pub(crate) fn apply_pagination<'a, ST, QS, DB>(
    source: diesel::query_builder::BoxedSelectStatement<'a, ST, QS, DB>,
    pagination: &Pagination,
) -> diesel::query_builder::BoxedSelectStatement<'a, ST, QS, DB>
where
    QS: diesel::query_source::QuerySource,
    DB: diesel::backend::Backend + diesel::sql_types::HasSqlType<ST> + 'a,
{
    let mut target = source;
    let Pagination { limit, offset } = pagination;
    let limit = limit.to_i64().unwrap_or(i64::MAX);
    target = target.limit(limit);
    if let Some(offset) = offset {
        // TODO: Verify that this restriction still applies!
        // SQLite: OFFSET can only be used in conjunction with LIMIT
        let offset = offset.to_i64().unwrap_or(i64::MAX);
        target = target.offset(offset);
    };
    target
}

pub enum StringCmpOp {
    Equal(String),
    Prefix(String, usize),
    Like(String),
}

pub const LIKE_ESCAPE_CHARACTER: char = '\\';

pub const LIKE_WILDCARD_CHARACTER: char = '%';
pub const LIKE_PLACEHOLDER_CHARACTER: char = '_';

const LIKE_ESCAPE_CHARACTER_REPLACEMENT: &str = "\\\\"; // LIKE_ESCAPE_CHARACTER + LIKE_ESCAPE_CHARACTER

const LIKE_WILDCARD_CHARACTER_REPLACEMENT: &str = "\\%"; // LIKE_ESCAPE_CHARACTER + LIKE_WILDCARD_CHARACTER
const LIKE_PLACEHOLDER_CHARACTER_REPLACEMENT: &str = "\\_"; // LIKE_ESCAPE_CHARACTER + LIKE_PLACEHOLDER_CHARACTER

pub fn escape_like_matches(arg: &str) -> String {
    // The order if replacements matters!
    arg.replace(LIKE_ESCAPE_CHARACTER, LIKE_ESCAPE_CHARACTER_REPLACEMENT)
        .replace(LIKE_WILDCARD_CHARACTER, LIKE_WILDCARD_CHARACTER_REPLACEMENT)
        .replace(
            LIKE_PLACEHOLDER_CHARACTER,
            LIKE_PLACEHOLDER_CHARACTER_REPLACEMENT,
        )
}

pub fn escape_single_quotes(arg: &str) -> String {
    arg.replace('\'', "''")
}

pub fn escape_like_starts_with(arg: &str) -> String {
    format!("{}{}", escape_like_matches(arg), LIKE_WILDCARD_CHARACTER)
}

pub fn escape_like_ends_with(arg: &str) -> String {
    format!("{}{}", LIKE_WILDCARD_CHARACTER, escape_like_matches(arg))
}

pub fn escape_like_contains(arg: &str) -> String {
    format!(
        "{}{}{}",
        LIKE_WILDCARD_CHARACTER,
        escape_like_matches(arg),
        LIKE_WILDCARD_CHARACTER
    )
}
