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

table! {
    collection_entity (id) {
        id -> BigInt,
        uid -> Text,
        rev_ordinal -> BigInt,
        rev_timestamp -> Timestamp,
        name -> Text,
        description -> Nullable<Text>,
    }
}

table! {
    active_collection (id) {
        id -> BigInt,
        collection_id -> BigInt,
    }
}