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

use aoide_core::{
    audio::DurationMs,
    media::Content,
    track::{Entity as TrackEntity, Track},
};

use aoide_repo::{
    collection::RecordId as CollectionId,
    track::{
        DateTimeField, DateTimeFieldFilter, EntityRepo as TrackRepo, NumericField,
        NumericFieldFilter, PhraseFieldFilter, RecordHeader, RecordId as TrackId, SearchFilter,
        SortField, SortOrder, StringField,
    },
};

use bitflags::bitflags;
use std::num::NonZeroUsize;

bitflags! {
    /// A bitmask for controlling how and if content metadata is
    /// re-imported from the source.
    pub struct SearchFlags: u8 {
        const NONE           = 0b00000000; // least restrictive
        const SOURCE_TRACKED = 0b00000001;
        const ALBUM_ARTIST   = 0b00000010;
        const ALBUM_TITLE    = 0b00000100;
        const TRACK_ARTIST   = 0b00001000;
        const TRACK_TITLE    = 0b00010000;
        const RELEASED_AT    = 0b00100000;
        const ALL            = 0b00111111; // most restrictive
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Params {
    pub audio_duration_tolerance: DurationMs,
    pub max_results: NonZeroUsize,
    pub search_flags: SearchFlags,
}

impl Params {
    pub const fn new() -> Params {
        // More than one result is necessary to decide if it is unambiguous
        let max_results = unsafe { NonZeroUsize::new_unchecked(2) };
        Self {
            audio_duration_tolerance: DurationMs::from_inner(500.0), // +/- 500 ms
            max_results,
            search_flags: SearchFlags::ALL,
        }
    }

    pub const fn with_max_results(max_results: NonZeroUsize) -> Params {
        Self {
            max_results,
            ..Self::new()
        }
    }
}

impl Default for Params {
    fn default() -> Self {
        Self::new()
    }
}

pub fn find_duplicate<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    track_id: TrackId,
    track: Track,
    params: &Params,
) -> RepoResult<Vec<(TrackId, TrackEntity)>>
where
    Repo: TrackRepo,
{
    let Params {
        audio_duration_tolerance,
        search_flags,
        max_results,
    } = params;
    let mut all_filters = Vec::with_capacity(10);
    if search_flags.contains(SearchFlags::TRACK_ARTIST) {
        if let Some(track_artist) = track.track_artist() {
            let track_artist = track_artist.trim();
            if !track_artist.is_empty() {
                all_filters.push(SearchFilter::Phrase(PhraseFieldFilter {
                    fields: vec![StringField::TrackArtist],
                    terms: vec![track_artist.to_owned()],
                }));
            }
        }
    }
    if search_flags.contains(SearchFlags::TRACK_TITLE) {
        if let Some(track_title) = track.track_title() {
            let track_title = track_title.trim();
            if !track_title.is_empty() {
                all_filters.push(SearchFilter::Phrase(PhraseFieldFilter {
                    fields: vec![StringField::TrackTitle],
                    terms: vec![track_title.to_owned()],
                }));
            }
        }
    }
    if search_flags.contains(SearchFlags::ALBUM_ARTIST) {
        if let Some(album_artist) = track.album_artist() {
            let album_artist = album_artist.trim();
            if !album_artist.is_empty() {
                all_filters.push(SearchFilter::Phrase(PhraseFieldFilter {
                    fields: vec![StringField::AlbumArtist],
                    terms: vec![album_artist.to_owned()],
                }));
            }
        }
    }
    if search_flags.contains(SearchFlags::ALBUM_TITLE) {
        if let Some(album_title) = track.album_title() {
            let album_title = album_title.trim();
            if !album_title.is_empty() {
                all_filters.push(SearchFilter::Phrase(PhraseFieldFilter {
                    fields: vec![StringField::AlbumTitle],
                    terms: vec![album_title.to_owned()],
                }));
            }
        }
    }
    if search_flags.contains(SearchFlags::RELEASED_AT) {
        all_filters.push(if let Some(released_at) = track.release.released_at {
            SearchFilter::released_at_equals(released_at)
        } else {
            SearchFilter::DateTime(DateTimeFieldFilter {
                field: DateTimeField::ReleasedAt,
                predicate: DateTimePredicate::Equal(None),
            })
        });
    }
    if search_flags.contains(SearchFlags::SOURCE_TRACKED) {
        all_filters.push(SearchFilter::Condition(
            aoide_repo::track::ConditionFilter::SourceTracked,
        ));
    }
    // Only sources with similar audio duration
    let audio_duration_ms = match track.media_source.content {
        Content::Audio(content) => content.duration,
    };
    all_filters.push(if let Some(audio_duration_ms) = audio_duration_ms {
        SearchFilter::audio_duration_around(audio_duration_ms, *audio_duration_tolerance)
    } else {
        SearchFilter::Numeric(NumericFieldFilter {
            field: NumericField::AudioDurationMs,
            predicate: NumericPredicate::Equal(None),
        })
    });
    // Only sources with equal content/file type
    all_filters.push(SearchFilter::Phrase(PhraseFieldFilter {
        fields: vec![StringField::SourceType],
        terms: vec![track.media_source.content_type],
    }));
    let filter = SearchFilter::All(all_filters);
    // Prefer recently added sources, e.g. after scanning the file system
    let ordering = vec![SortOrder {
        field: SortField::SourceCollectedAt,
        direction: SortDirection::Descending,
    }];
    let mut candidates = Vec::new();
    repo.search_collected_tracks(
        collection_id,
        &Default::default(),
        Some(filter),
        ordering,
        &mut candidates,
    )?;
    Ok(candidates
        .into_iter()
        .filter_map(|(record_header, entity)| {
            if record_header.id == track_id {
                // Exclude the track if contained in the search results
                None
            } else {
                Some((record_header.id, entity))
            }
        })
        .take(max_results.get())
        .collect())
}

pub fn find_duplicate_by_media_source_path<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    media_source_path: &str,
    params: &Params,
) -> RepoResult<Vec<(TrackId, TrackEntity)>>
where
    Repo: TrackRepo,
{
    let (_media_source_id, RecordHeader { id: track_id, .. }, entity) =
        repo.load_track_entity_by_media_source_path(collection_id, media_source_path)?;
    find_duplicate(repo, collection_id, track_id, entity.body, params)
}
