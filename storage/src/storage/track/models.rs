// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::schema::*;

use chrono::{
    naive::{NaiveDate, NaiveDateTime},
    DateTime, Utc,
};

use percent_encoding::percent_decode;

use api::{
    entity::StorageId,
    serde::{SerializationFormat, SerializedEntity},
};

use aoide_core::domain::entity::{EntityHeader, EntityRevision, EntityUid, EntityVersion};
use aoide_core::domain::metadata::{Comment, Rating, Score, ScoreValue};
use aoide_core::domain::music::notation::Beats;
use aoide_core::domain::music::{ActorRole, Actors, SongFeature, SongProfile, TitleLevel, Titles};
use aoide_core::domain::track::{RefOrigin, Track, TrackCollection, TrackSource};

#[derive(Debug, Insertable)]
#[table_name = "tbl_track"]
pub struct InsertableTracksEntity<'a> {
    pub uid: &'a [u8],
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub ser_fmt: i16,
    pub ser_ver_major: i32,
    pub ser_ver_minor: i32,
    pub ser_blob: &'a [u8],
}

impl<'a> InsertableTracksEntity<'a> {
    pub fn bind(
        header: &'a EntityHeader,
        ser_fmt: SerializationFormat,
        ser_blob: &'a [u8],
    ) -> Self {
        Self {
            uid: header.uid().as_ref(),
            rev_ordinal: header.revision().ordinal() as i64,
            rev_timestamp: header.revision().timestamp().naive_utc(),
            ser_fmt: ser_fmt as i16,
            ser_ver_major: 0, // TODO
            ser_ver_minor: 0, // TODO
            ser_blob,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[table_name = "tbl_track"]
pub struct UpdatableTracksEntity<'a> {
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub ser_fmt: i16,
    pub ser_ver_major: i32,
    pub ser_ver_minor: i32,
    pub ser_blob: &'a [u8],
}

impl<'a> UpdatableTracksEntity<'a> {
    pub fn bind(
        next_revision: &'a EntityRevision,
        ser_fmt: SerializationFormat,
        ser_blob: &'a [u8],
    ) -> Self {
        Self {
            rev_ordinal: next_revision.ordinal() as i64,
            rev_timestamp: next_revision.timestamp().naive_utc(),
            ser_fmt: ser_fmt as i16,
            ser_ver_major: 0, // TODO
            ser_ver_minor: 0, // TODO
            ser_blob,
        }
    }
}

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "tbl_track"]
pub struct QueryableSerializedEntity {
    pub id: StorageId,
    pub uid: Vec<u8>,
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub ser_fmt: i16,
    pub ser_ver_major: i32,
    pub ser_ver_minor: i32,
    pub ser_blob: Vec<u8>,
}

impl From<QueryableSerializedEntity> for SerializedEntity {
    fn from(from: QueryableSerializedEntity) -> Self {
        let uid = EntityUid::from_slice(&from.uid);
        let revision = EntityRevision::new(
            from.rev_ordinal as u64,
            DateTime::from_utc(from.rev_timestamp, Utc),
        );
        let header = EntityHeader::new(uid, revision);
        let format = SerializationFormat::from(from.ser_fmt).unwrap();
        debug_assert!(from.ser_ver_major >= 0);
        debug_assert!(from.ser_ver_minor >= 0);
        let version = EntityVersion::new(from.ser_ver_major as u32, from.ser_ver_minor as u32);
        SerializedEntity {
            header,
            format,
            version,
            blob: from.ser_blob,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_overview"]
pub struct InsertableTracksOverview<'a> {
    pub track_id: StorageId,
    pub track_title: Option<&'a str>,
    pub track_subtitle: Option<&'a str>,
    pub track_work: Option<&'a str>,
    pub track_movement: Option<&'a str>,
    pub album_title: Option<&'a str>,
    pub album_subtitle: Option<&'a str>,
    pub released_at: Option<NaiveDate>,
    pub released_by: Option<&'a str>,
    pub release_copyright: Option<&'a str>,
    pub track_index: Option<i32>,
    pub track_count: Option<i32>,
    pub disc_index: Option<i32>,
    pub disc_count: Option<i32>,
    pub movement_index: Option<i32>,
    pub movement_count: Option<i32>,
    pub lyrics_explicit: Option<bool>,
    pub album_compilation: Option<bool>,
}

impl<'a> InsertableTracksOverview<'a> {
    pub fn bind(track_id: StorageId, track: &'a Track) -> Self {
        Self {
            track_id,
            track_title: Titles::main_title(&track.titles).map(|title| title.name.as_str()),
            track_subtitle: Titles::title(&track.titles, TitleLevel::Sub, None)
                .map(|title| title.name.as_str()),
            track_work: Titles::title(&track.titles, TitleLevel::Work, None)
                .map(|title| title.name.as_str()),
            track_movement: Titles::title(&track.titles, TitleLevel::Movement, None)
                .map(|title| title.name.as_str()),
            album_title: track
                .album
                .as_ref()
                .and_then(|album| Titles::main_title(&album.titles))
                .map(|title| title.name.as_str()),
            album_subtitle: track
                .album
                .as_ref()
                .and_then(|album| Titles::title(&album.titles, TitleLevel::Sub, None))
                .map(|title| title.name.as_str()),
            released_at: track
                .release
                .as_ref()
                .and_then(|release| release.released_at)
                .map(|released_at| released_at.date().naive_utc()),
            released_by: track
                .release
                .as_ref()
                .and_then(|release| release.released_by.as_ref())
                .map(|released_by| released_by.as_str()),
            release_copyright: track
                .release
                .as_ref()
                .and_then(|release| release.copyright.as_ref())
                .map(|copyright| copyright.as_str()),
            track_index: track.track_numbers.index().map(|index| index as i32),
            track_count: track.track_numbers.count().map(|count| count as i32),
            disc_index: track.disc_numbers.index().map(|index| index as i32),
            disc_count: track.disc_numbers.count().map(|count| count as i32),
            movement_index: track.movement_numbers.index().map(|index| index as i32),
            movement_count: track.movement_numbers.count().map(|count| count as i32),
            lyrics_explicit: track.lyrics.as_ref().and_then(|lyrics| lyrics.explicit),
            album_compilation: track.album.as_ref().and_then(|album| album.compilation),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_summary"]
pub struct InsertableTracksSummary<'a> {
    pub track_id: StorageId,
    pub track_artist: Option<&'a str>,
    pub track_composer: Option<&'a str>,
    pub track_conductor: Option<&'a str>,
    pub track_performer: Option<&'a str>,
    pub track_producer: Option<&'a str>,
    pub track_remixer: Option<&'a str>,
    pub album_artist: Option<&'a str>,
    pub album_composer: Option<&'a str>,
    pub album_conductor: Option<&'a str>,
    pub album_performer: Option<&'a str>,
    pub album_producer: Option<&'a str>,
    pub ratings_min: Option<ScoreValue>,
    pub ratings_max: Option<ScoreValue>,
}

impl<'a> InsertableTracksSummary<'a> {
    pub fn bind(track_id: StorageId, track: &'a Track) -> Self {
        let (ratings_min, ratings_max) = match Rating::minmax(&track.ratings, None) {
            Some((Score(min), Score(max))) => (Some(min), Some(max)),
            None => (None, None),
        };
        Self {
            track_id,
            track_artist: Actors::main_actor(&track.actors, ActorRole::Artist)
                .map(|actor| actor.name.as_str()),
            track_composer: Actors::main_actor(&track.actors, ActorRole::Composer)
                .map(|actor| actor.name.as_str()),
            track_conductor: Actors::main_actor(&track.actors, ActorRole::Conductor)
                .map(|actor| actor.name.as_str()),
            track_performer: Actors::main_actor(&track.actors, ActorRole::Performer)
                .map(|actor| actor.name.as_str()),
            track_producer: Actors::main_actor(&track.actors, ActorRole::Producer)
                .map(|actor| actor.name.as_str()),
            track_remixer: Actors::main_actor(&track.actors, ActorRole::Remixer)
                .map(|actor| actor.name.as_str()),
            album_artist: track
                .album
                .as_ref()
                .and_then(|album| Actors::main_actor(&album.actors, ActorRole::Artist))
                .map(|actor| actor.name.as_str()),
            album_composer: track
                .album
                .as_ref()
                .and_then(|album| Actors::main_actor(&album.actors, ActorRole::Composer))
                .map(|actor| actor.name.as_str()),
            album_conductor: track
                .album
                .as_ref()
                .and_then(|album| Actors::main_actor(&album.actors, ActorRole::Conductor))
                .map(|actor| actor.name.as_str()),
            album_performer: track
                .album
                .as_ref()
                .and_then(|album| Actors::main_actor(&album.actors, ActorRole::Performer))
                .map(|actor| actor.name.as_str()),
            album_producer: track
                .album
                .as_ref()
                .and_then(|album| Actors::main_actor(&album.actors, ActorRole::Producer))
                .map(|actor| actor.name.as_str()),
            ratings_min,
            ratings_max,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_source"]
pub struct InsertableTracksSource<'a> {
    pub track_id: StorageId,
    pub content_uri: &'a str,
    pub content_uri_decoded: String,
    pub content_type: &'a str,
    pub audio_channels_count: Option<i16>,
    pub audio_duration_ms: Option<f64>,
    pub audio_samplerate_hz: Option<i32>,
    pub audio_bitrate_bps: Option<i32>,
    pub audio_enc_name: Option<&'a str>,
    pub audio_enc_settings: Option<&'a str>,
    pub metadata_sync_when: Option<NaiveDateTime>,
    pub metadata_sync_rev_ordinal: Option<i64>,
    pub metadata_sync_rev_timestamp: Option<NaiveDateTime>,
}

impl<'a> InsertableTracksSource<'a> {
    pub fn bind(track_id: StorageId, track_source: &'a TrackSource) -> Self {
        Self {
            track_id,
            content_uri: track_source.content_uri.as_str(),
            content_uri_decoded: percent_decode(track_source.content_uri.as_bytes())
                .decode_utf8_lossy()
                .into(),
            content_type: track_source.content_type.as_str(),
            audio_channels_count: track_source
                .audio_content
                .as_ref()
                .map(|audio| *audio.channels.count as i16),
            audio_duration_ms: track_source
                .audio_content
                .as_ref()
                .map(|audio| audio.duration.ms()),
            audio_samplerate_hz: track_source
                .audio_content
                .as_ref()
                .map(|audio| audio.sample_rate.hz() as i32),
            audio_bitrate_bps: track_source
                .audio_content
                .as_ref()
                .map(|audio| audio.bit_rate.bps() as i32),
            audio_enc_name: track_source
                .audio_content
                .as_ref()
                .and_then(|audio| audio.encoder.as_ref())
                .map(|enc| enc.name.as_str()),
            audio_enc_settings: track_source
                .audio_content
                .as_ref()
                .and_then(|audio| audio.encoder.as_ref())
                .and_then(|enc| enc.settings.as_ref())
                .map(|settings| settings.as_str()),
            metadata_sync_when: track_source.metadata_sync.map(|sync| sync.when.naive_utc()),
            metadata_sync_rev_ordinal: track_source
                .metadata_sync
                .map(|sync| sync.revision.ordinal() as i64),
            metadata_sync_rev_timestamp: track_source
                .metadata_sync
                .map(|sync| sync.revision.timestamp().naive_utc()),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_collection"]
pub struct InsertableTracksCollection<'a> {
    pub track_id: StorageId,
    pub collection_uid: &'a [u8],
    pub since: NaiveDateTime,
    pub color_code: Option<i32>,
    pub play_count: Option<i32>,
}

impl<'a> InsertableTracksCollection<'a> {
    pub fn bind(track_id: StorageId, track_collection: &'a TrackCollection) -> Self {
        Self {
            track_id,
            collection_uid: track_collection.uid.as_ref(),
            since: track_collection.since.naive_utc(),
            color_code: track_collection.color.map(|color| color.code() as i32),
            play_count: track_collection.play_count.map(|count| count as i32),
        }
    }
}

#[derive(Debug, Clone, Copy, Insertable)]
#[table_name = "aux_track_profile"]
pub struct InsertableTracksMusic {
    pub track_id: StorageId,
    pub tempo_bpm: Beats,
    pub time_sig_top: i16,
    pub time_sig_bottom: i16,
    pub key_sig_code: i16,
    pub acousticness_score: Option<ScoreValue>,
    pub danceability_score: Option<ScoreValue>,
    pub energy_score: Option<ScoreValue>,
    pub instrumentalness_score: Option<ScoreValue>,
    pub liveness_score: Option<ScoreValue>,
    pub popularity_score: Option<ScoreValue>,
    pub speechiness_score: Option<ScoreValue>,
    pub valence_score: Option<ScoreValue>,
}

impl InsertableTracksMusic {
    pub fn bind(track_id: StorageId, profile: &SongProfile) -> Self {
        Self {
            track_id,
            tempo_bpm: profile.tempo.bpm(),
            time_sig_top: profile.time_sig.top() as i16,
            time_sig_bottom: profile.time_sig.bottom() as i16,
            key_sig_code: i16::from(profile.key_sig.code()),
            acousticness_score: profile
                .feature(SongFeature::Acousticness)
                .map(|feature_score| *feature_score.score()),
            danceability_score: profile
                .feature(SongFeature::Danceability)
                .map(|feature_score| *feature_score.score()),
            energy_score: profile
                .feature(SongFeature::Energy)
                .map(|feature_score| *feature_score.score()),
            instrumentalness_score: profile
                .feature(SongFeature::Instrumentalness)
                .map(|feature_score| *feature_score.score()),
            liveness_score: profile
                .feature(SongFeature::Liveness)
                .map(|feature_score| *feature_score.score()),
            popularity_score: profile
                .feature(SongFeature::Popularity)
                .map(|feature_score| *feature_score.score()),
            speechiness_score: profile
                .feature(SongFeature::Speechiness)
                .map(|feature_score| *feature_score.score()),
            valence_score: profile
                .feature(SongFeature::Valence)
                .map(|feature_score| *feature_score.score()),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_xref"]
pub struct InsertableTracksRef<'a> {
    pub track_id: StorageId,
    pub origin: i16,
    pub reference: &'a str,
}

impl<'a> InsertableTracksRef<'a> {
    pub fn bind(track_id: StorageId, origin: RefOrigin, reference: &'a str) -> Self {
        Self {
            track_id,
            origin: origin as i16,
            reference,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_tag_term"]
pub struct InsertableTracksTagTerm<'a> {
    pub term: &'a str,
}

impl<'a> InsertableTracksTagTerm<'a> {
    pub fn bind(term: &'a str) -> Self {
        Self { term }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_tag_facet"]
pub struct InsertableTracksTagFacet<'a> {
    pub facet: &'a str,
}

impl<'a> InsertableTracksTagFacet<'a> {
    pub fn bind(facet: &'a str) -> Self {
        Self { facet }
    }
}

#[derive(Debug, Clone, Copy, Insertable)]
#[table_name = "aux_track_tag"]
pub struct InsertableTracksTag {
    pub track_id: StorageId,
    pub term_id: StorageId,
    pub facet_id: Option<StorageId>,
    pub score: ScoreValue,
}

impl InsertableTracksTag {
    pub fn bind(
        track_id: StorageId,
        term_id: StorageId,
        facet_id: Option<StorageId>,
        score: Score,
    ) -> Self {
        Self {
            track_id,
            term_id,
            facet_id,
            score: *score,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_rating"]
pub struct InsertableTracksRating<'a> {
    pub track_id: StorageId,
    pub score: ScoreValue,
    pub owner: Option<&'a str>,
}

impl<'a> InsertableTracksRating<'a> {
    pub fn bind(track_id: StorageId, rating: &'a Rating) -> Self {
        Self {
            track_id,
            score: *rating.score(),
            owner: rating.owner().as_ref().map(|owner| owner.as_str()),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_comment"]
pub struct InsertableTracksComment<'a> {
    pub track_id: StorageId,
    pub text: &'a str,
    pub owner: Option<&'a str>,
}

impl<'a> InsertableTracksComment<'a> {
    pub fn bind(track_id: StorageId, comment: &'a Comment) -> Self {
        Self {
            track_id,
            text: comment.text().as_str(),
            owner: comment.owner().as_ref().map(|owner| owner.as_str()),
        }
    }
}