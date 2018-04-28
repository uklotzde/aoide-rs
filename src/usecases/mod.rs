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

use failure::Error;

use aoide_core::domain::entity::*;
use aoide_core::domain::collection::*;

#[derive(Debug, Clone, Copy, Fail, PartialEq, Eq)]
pub enum CollectionsError {
    #[fail(display = "Collections: Not found")]
    NotFound,
    #[fail(display = "Collections: Unexpected error")]
    Unexpected,
}

pub type CollectionsResult<T> = Result<T, CollectionsError>;

pub trait Collections {
    fn create_entity(&self, body: CollectionBody) -> CollectionsResult<CollectionEntity>;

    fn update_entity(&self, entity: &mut CollectionEntity) -> CollectionsResult<EntityRevision>;

    fn find_entity(&self, uid: &EntityUid) -> CollectionsResult<Option<CollectionEntity>>;

    fn remove_entity(&self, uid: &EntityUid) -> CollectionsResult<()>;

    fn find_entities_by_name(&self, name: &str) -> CollectionsResult<Vec<CollectionEntity>>;

    fn find_entities_by_name_starting_with(
        &self,
        name: &str,
    ) -> CollectionsResult<Vec<CollectionEntity>>;

    fn find_entities_by_name_containing(
        &self,
        name: &str,
    ) -> CollectionsResult<Vec<CollectionEntity>>;

    fn activate_collection(&self, uid: &EntityUid) -> CollectionsResult<()>;
}