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

use crate::{collection, entity::*, tag};

use aoide_core::{
    collection::SingleTrackEntry as CollectionSingleTrackEntry,
    entity::{EntityRevisionUpdateResult, EntityUid},
    track::{album::*, *},
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StringField {
    MediaUri,        // percent-encoded URI
    MediaUriDecoded, // percent-decoded URI
    MediaType,
    TrackTitle,
    TrackArtist,
    TrackComposer,
    AlbumTitle,
    AlbumArtist,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NumericField {
    AudioBitRate,
    AudioChannelCount,
    AudioDuration,
    AudioSampleRate,
    AudioLoudness,
    TrackNumber,
    TrackTotal,
    DiscNumber,
    DiscTotal,
    ReleaseDate,
    MusicBpm,
    MusicKey,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NumericFieldFilter {
    pub field: NumericField,
    pub value: NumericPredicate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhraseFieldFilter {
    // Empty == All available string fields are considered
    // Disjunction, i.e. a match in one of the fields is sufficient
    pub fields: Vec<StringField>,

    // Concatenated with wildcards and filtered using
    // case-insensitive "contains" semantics against each
    // of the selected fields, e.g. ["pa", "la", "bell"]
    // ["tt, ll"] will both match "Patti LaBelle". An empty
    // argument matches empty as well as missing/null fields.
    pub terms: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaSourceFilterParams {
    pub media_uri: StringPredicate,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SortField {
    InCollectionSince, // = recently added (only if searching in a single collection)
    LastRevisionedAt,  // = recently modified (created or updated)
    MediaUri,          // percent-encoded URI
    MediaUriDecoded,   // plain URI
    TrackTitle,
    TrackArtist,
    TrackNumber,
    TrackTotal,
    DiscNumber,
    DiscTotal,
    AlbumTitle,
    AlbumArtist,
    ReleaseDate,
    MusicBpm,
    MusicKey,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SortOrder {
    pub field: SortField,
    pub direction: SortDirection,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SearchFilter {
    Phrase(PhraseFieldFilter),
    Numeric(NumericFieldFilter),
    Tag(tag::Filter),
    MarkerLabel(StringFilter),
    All(Vec<SearchFilter>),
    Any(Vec<SearchFilter>),
    Not(Box<SearchFilter>),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchParams {
    pub filter: Option<SearchFilter>,
    pub ordering: Vec<SortOrder>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringFieldCounts {
    pub field: StringField,
    pub counts: Vec<StringCount>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReplaceMode {
    UpdateOnly,
    UpdateOrCreate,
}

// Successful outcomes that allow batch processing and
// handle conflicts on an outer level. Only technical
// failures are considered as errors!
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplaceOutcome {
    AmbiguousMediaUri(usize),
    IncompatibleFormat(EntityDataFormat),
    IncompatibleVersion(EntityDataVersion),
    NotCreated,
    Unchanged(EntityHeader),
    Created(EntityHeader),
    Updated(EntityHeader),
}

pub fn collect_entries_from_rows<T, R>(
    rows: Vec<T>,
    collection_uid: &EntityUid,
    collection_entry_repo: &R,
) -> RepoResult<Vec<EntityDataExt<Option<CollectionSingleTrackEntry>>>>
where
    T: Into<EntityData>,
    R: collection::TrackEntryRepo,
{
    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let entity_data = row.into();
        let track_uid = &entity_data.0.uid;
        let collection_entry = collection_entry_repo.load_track_entry(collection_uid, track_uid)?;
        entries.push((entity_data, collection_entry));
    }
    Ok(entries)
}

pub trait Repo {
    fn resolve_track_id(&self, uid: &EntityUid) -> RepoResult<Option<RepoId>>;

    fn insert_track(
        &self,
        collection_uid: Option<&EntityUid>,
        entity: Entity,
        body_data: EntityBodyData,
    ) -> RepoResult<()>;

    fn update_track(
        &self,
        collection_uid: Option<&EntityUid>,
        entity: Entity,
        body_data: EntityBodyData,
    ) -> RepoResult<EntityRevisionUpdateResult>;

    fn delete_track(&self, uid: &EntityUid) -> RepoResult<Option<()>>;

    /// Load a single track by UID.
    fn load_track(&self, uid: &EntityUid) -> RepoResult<Option<EntityData>>;

    /// Load multiple tracks by their UID.
    ///
    /// The result may contain fewer tracks than requested if some
    /// tracks do not exist. The order of the given UIDs is not preserved
    /// in the result set, i.e. the ordering of tracks is undefined!!
    fn load_tracks(&self, uids: &[EntityUid]) -> RepoResult<Vec<EntityData>>;

    fn locate_tracks(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: Pagination,
        filter_params: MediaSourceFilterParams,
    ) -> RepoResult<Vec<EntityData>>;

    fn locate_tracks_in_collection(
        &self,
        collection_uid: &EntityUid,
        pagination: Pagination,
        filter_params: MediaSourceFilterParams,
    ) -> RepoResult<Vec<EntityDataExt<Option<CollectionSingleTrackEntry>>>>;

    fn resolve_tracks_by_media_source_uri(
        &self,
        collection_uid: &EntityUid, // for disambiguation
        media_uris: &[String],
    ) -> RepoResult<Vec<(String, EntityUid)>>;

    fn search_tracks(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: Pagination,
        search_params: SearchParams,
    ) -> RepoResult<Vec<EntityData>>;

    fn search_tracks_in_collection(
        &self,
        collection_uid: &EntityUid,
        pagination: Pagination,
        search_params: SearchParams,
    ) -> RepoResult<Vec<EntityDataExt<Option<CollectionSingleTrackEntry>>>>;

    fn count_track_field_strings(
        &self,
        collection_uid: Option<&EntityUid>,
        field: StringField,
        pagination: Pagination,
    ) -> RepoResult<StringFieldCounts>;

    fn collect_collection_track_stats(
        &self,
        collection_uid: &EntityUid,
    ) -> RepoResult<collection::TrackStats>;

    fn replace_track(
        &self,
        collection_uid: Option<&EntityUid>,
        media_uri: String,
        mode: ReplaceMode,
        track: Track,
        body_data: EntityBodyData,
    ) -> RepoResult<(ReplaceOutcome, Option<CollectionSingleTrackEntry>)> {
        let locate_params = MediaSourceFilterParams {
            media_uri: StringPredicate::Equals(media_uri),
        };
        let (entity_data, collection_entry) = if let Some(collection_uid) = collection_uid {
            let located_tracks = self.locate_tracks_in_collection(
                collection_uid,
                Pagination::default(),
                locate_params,
            )?;
            if located_tracks.len() > 1 {
                return Ok((
                    ReplaceOutcome::AmbiguousMediaUri(located_tracks.len()),
                    None,
                ));
            }
            located_tracks
                .into_iter()
                .next()
                .map(|(a, b)| (Some(a), b))
                .unwrap_or((None, None))
        } else {
            let located_tracks =
                self.locate_tracks(collection_uid, Pagination::default(), locate_params)?;
            if located_tracks.len() > 1 {
                return Ok((
                    ReplaceOutcome::AmbiguousMediaUri(located_tracks.len()),
                    None,
                ));
            }
            located_tracks
                .into_iter()
                .next()
                .map(|item| (Some(item), None))
                .unwrap_or((None, None))
        };
        let (data_fmt, data_ver, data_blob) = body_data;
        if let Some((entity_hdr, (entity_fmt, entity_ver, entity_blob))) = entity_data {
            // Update
            if entity_fmt != data_fmt {
                return Ok((
                    ReplaceOutcome::IncompatibleFormat(entity_fmt),
                    collection_entry,
                ));
            }
            if entity_ver != data_ver {
                return Ok((
                    ReplaceOutcome::IncompatibleVersion(entity_ver),
                    collection_entry,
                ));
            }
            if entity_blob == data_blob {
                return Ok((ReplaceOutcome::Unchanged(entity_hdr), collection_entry));
            }
            let old_hdr = entity_hdr;
            let entity = Entity::new(old_hdr.clone(), track);
            match self.update_track(collection_uid, entity, (data_fmt, data_ver, data_blob))? {
                EntityRevisionUpdateResult::NotFound => {
                    bail!("Failed to update track {:?}: Not found", old_hdr);
                }
                EntityRevisionUpdateResult::Current(rev) => {
                    bail!(
                        "Failed to update track {:?}: Current revision {:?} is newer",
                        old_hdr,
                        rev,
                    );
                }
                EntityRevisionUpdateResult::Updated(_, rev) => {
                    let uid = old_hdr.uid;
                    let new_hdr = EntityHeader { uid, rev };
                    Ok((ReplaceOutcome::Updated(new_hdr), collection_entry))
                }
            }
        } else {
            // Create
            match mode {
                ReplaceMode::UpdateOnly => Ok((ReplaceOutcome::NotCreated, None)),
                ReplaceMode::UpdateOrCreate => {
                    let hdr = EntityHeader::initial_random();
                    let entity = Entity::new(hdr.clone(), track);
                    self.insert_track(collection_uid, entity, (data_fmt, data_ver, data_blob))?;
                    Ok((ReplaceOutcome::Created(hdr), None))
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CountTracksByAlbumParams {
    pub min_release_date: Option<Date>,
    pub max_release_date: Option<Date>,

    pub ordering: Vec<SortOrder>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AlbumCountResults {
    pub title: Option<String>,

    pub artist: Option<String>,

    pub release_date: Option<Date>,

    pub total_count: usize,
}

impl AlbumCountResults {
    pub fn new_for_album(
        album: &Album,
        release_date: impl Into<Option<Date>>,
        total_count: usize,
    ) -> Self {
        let title = album.main_title().map(|title| title.name.to_string());
        let artist = album.main_artist().map(|actor| actor.name.to_string());
        let release_date = release_date.into();
        Self {
            title,
            artist,
            release_date,
            total_count,
        }
    }
}

pub trait Albums {
    fn count_tracks_by_album(
        &self,
        collection_uid: Option<&EntityUid>,
        params: &CountTracksByAlbumParams,
        pagination: Pagination,
    ) -> RepoResult<Vec<AlbumCountResults>>;
}

pub trait Tags {
    fn count_tracks_by_tag_facet(
        &self,
        collection_uid: Option<&EntityUid>,
        params: &tag::FacetCountParams,
        pagination: Pagination,
    ) -> RepoResult<Vec<tag::FacetCount>>;

    fn count_tracks_by_tag(
        &self,
        collection_uid: Option<&EntityUid>,
        params: &tag::CountParams,
        pagination: Pagination,
    ) -> RepoResult<Vec<tag::AvgScoreCount>>;
}
