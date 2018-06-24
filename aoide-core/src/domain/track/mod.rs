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

#[cfg(test)]
mod tests;

use audio::sample::*;
use audio::signal::*;
use audio::*;
use domain::entity::*;
use domain::metadata::*;
use domain::music::*;

use chrono::{DateTime, Utc};

use std::fmt;

///////////////////////////////////////////////////////////////////////
/// AudioEncoder
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AudioEncoder {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<String>,
}

impl AudioEncoder {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }
}

///////////////////////////////////////////////////////////////////////
/// AudioContent
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AudioContent {
    pub duration: Duration,

    pub channels: Channels,

    pub samplerate: SampleRate,

    pub bitrate: BitRate,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub loudness: Option<Loudness>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoder: Option<AudioEncoder>,
}

impl AudioContent {
    pub fn is_valid(&self) -> bool {
        !self.duration.is_empty()
            && self.channels.is_valid()
            && self.samplerate.is_valid()
            && self.bitrate.is_valid()
            && self.loudness.iter().all(Loudness::is_valid)
            && self.encoder.as_ref().map_or(true, |e| e.is_valid())
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackSource
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSynchronization {
    pub revision: EntityRevision,

    pub when: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSource {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub uri: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub synchronization: Option<TrackSynchronization>, // most recent metadata import/export

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub media_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_content: Option<AudioContent>,
}

impl TrackSource {
    pub fn is_valid(&self) -> bool {
        // TODO: Validate the URI
        // Currently (2018-05-28) there is no crate that is able to do this.
        // Crate http/hyper: Fail to recognize absolute file paths with the
        // scheme "file" and without an authority, e.g. parsing fails for
        // "file:///path/to/local/file.txt"
        // Crate url: Doesn't care about reserved characters, e.g. parses
        // "file:///path to local/file.txt" successfully
        !self.uri.is_empty() && !self.media_type.is_empty()
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackResource
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TrackCollection {
    pub uid: EntityUid,

    pub since: DateTime<Utc>,
}

impl TrackCollection {
    pub fn is_valid(&self) -> bool {
        self.uid.is_valid()
    }
}

pub type TrackColorCode = u32; // 0xAARRGGBB

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackColor {
    pub code: TrackColorCode,
}

impl TrackColor {
    pub const ALPHA_MASK: TrackColorCode = 0xff000000;
    pub const RED_MASK: TrackColorCode = 0x00ff0000;
    pub const GREEN_MASK: TrackColorCode = 0x0000ff00;
    pub const BLUE_MASK: TrackColorCode = 0x000000ff;

    pub const BLACK: Self = Self {
        code: Self::ALPHA_MASK,
    };
    pub const RED: Self = Self {
        code: Self::ALPHA_MASK | Self::RED_MASK,
    };
    pub const GREEN: Self = Self {
        code: Self::ALPHA_MASK | Self::GREEN_MASK,
    };
    pub const BLUE: Self = Self {
        code: Self::ALPHA_MASK | Self::BLUE_MASK,
    };
    pub const YELLOW: Self = Self {
        code: Self::ALPHA_MASK | Self::RED_MASK | Self::GREEN_MASK,
    };
    pub const MAGENTA: Self = Self {
        code: Self::ALPHA_MASK | Self::RED_MASK | Self::BLUE_MASK,
    };
    pub const CYAN: Self = Self {
        code: Self::ALPHA_MASK | Self::GREEN_MASK | Self::BLUE_MASK,
    };
    pub const WHITE: Self = Self {
        code: Self::ALPHA_MASK | Self::RED_MASK | Self::GREEN_MASK | Self::BLUE_MASK,
    };

    pub fn is_valid(&self) -> bool {
        true
    }

    pub fn into_opaque(&self) -> Self {
        Self {
            code: self.code | Self::ALPHA_MASK,
        }
    }

    pub fn into_transparent(&self) -> Self {
        Self {
            code: self.code & !Self::ALPHA_MASK,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackResource {
    pub collection: TrackCollection,

    pub source: TrackSource,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<TrackColor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_counter: Option<usize>,
}

impl TrackResource {
    pub fn is_valid(&self) -> bool {
        self.collection.is_valid()
            && self.source.is_valid()
            && self.color.iter().all(TrackColor::is_valid)
    }
}

///////////////////////////////////////////////////////////////////////
/// ReleaseMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReleaseMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub released_by: Option<String>, // record label

    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub licenses: Vec<String>,

    #[serde(rename = "xrefs", skip_serializing_if = "Vec::is_empty", default)]
    pub external_references: Vec<String>,
}

impl ReleaseMetadata {
    pub fn is_valid(&self) -> bool {
        true
    }
}

///////////////////////////////////////////////////////////////////////
/// AlbumMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AlbumMetadata {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub titles: Vec<Title>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub actors: Vec<Actor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub compilation: Option<bool>,

    #[serde(rename = "xrefs", skip_serializing_if = "Vec::is_empty", default)]
    pub external_references: Vec<String>,
}

impl AlbumMetadata {
    pub fn is_valid(&self) -> bool {
        Titles::is_valid(&self.titles) && Actors::is_valid(&self.actors)
    }
}

///////////////////////////////////////////////////////////////////////
/// IndexCount
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct IndexCount(/*index*/ Option<u32>, /*count*/ Option<u32>);

impl IndexCount {
    pub fn index(&self) -> Option<u32> {
        self.0
    }

    pub fn count(&self) -> Option<u32> {
        self.1
    }

    pub fn is_empty(&self) -> bool {
        self.index().is_none() && self.count().is_none()
    }

    pub fn is_valid(&self) -> bool {
        match (self.index(), self.count()) {
            (None, None) => true,
            (Some(index), None) => index > 0,
            (None, Some(count)) => count > 0,
            (Some(index), Some(count)) => index > 0 && index <= count,
        }
    }
}

impl fmt::Display for IndexCount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.index(), self.count()) {
            (None, None) => write!(f, ""),
            (Some(index), None) => write!(f, "{}", index),
            (None, Some(count)) => write!(f, "/{}", count),
            (Some(index), Some(count)) => write!(f, "{}/{}", index, count),
        }
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackMarker
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum TrackMark {
    // Cueing: Points without a length
    LoadCue, // default start point when loading a track, only one per track
    HotCue,
    // Fading: Short sections for automatic playback transitions
    FadeIn,  // only one per track
    FadeOut, // only one per track
    // Mixing: Long sections for manual transitions with beat matching
    MixIn,
    MixOut,
    // Sampling
    Sample,
    // Looping
    Loop,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackMarker {
    pub mark: TrackMark,

    pub offset: Duration,

    #[serde(skip_serializing_if = "Duration::is_empty", default)]
    pub length: Duration,

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub label: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<TrackColor>,
}

impl TrackMarker {
    pub fn is_singular(mark: TrackMark) -> bool {
        match mark {
            TrackMark::LoadCue | TrackMark::FadeIn | TrackMark::FadeOut => true,
            _ => false,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.offset.is_valid() && self.length.is_valid() && match self.mark {
            TrackMark::LoadCue | TrackMark::HotCue => self.length.is_empty(), // not available
            TrackMark::Sample | TrackMark::Loop => !self.length.is_empty(),   // mandatory
            _ => true, // optional, i.e. no restrictions on length
        }
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackTagging
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub struct TrackTagging;

impl TrackTagging {
    // Some predefined facets that are commonly used and could serve as a starting point

    // ISO 639-2 language codes: "eng", "fre"/"fra", "ita", "spa", "ger"/"deu", ...
    pub const FACET_LANG: &'static str = "lang";

    // The Content Group aka Grouping field
    pub const FACET_CGROUP: &'static str = "cgroup";

    // "Pop", "Dance", "Electronic", "R&B/Soul", "Hip Hop/Rap", ...
    pub const FACET_GENRE: &'static str = "genre";

    // Sub-genres or details like "East Coast", "West Coast", ...
    pub const FACET_STYLE: &'static str = "style";

    // "Happy", "Sexy", "Sad", "Melancholic", "Uplifting", ...
    pub const FACET_MOOD: &'static str = "mood";

    // Decades like "1980s", "2000s", ..., or other time-related feature
    pub const FACET_EPOCH: &'static str = "epoch";

    // "Birthday"/"Bday", "Xmas"/"Holiday"/"Christmas", "Summer", "Vacation", "Wedding", "Workout"...
    pub const FACET_EVENT: &'static str = "event";

    // "Bar", "Beach", "Dinner", "Club", "Lounge", ...
    pub const FACET_VENUE: &'static str = "venue";

    // "Dinner", "Festival", "Party", "Soundcheck", "Top40", "Workout", ...
    pub const FACET_CROWD: &'static str = "crowd";

    // "Warmup", "Opener", "Filler", "Peak", "Closer", "Afterhours", ...
    pub const FACET_SESSION: &'static str = "session";

    // Equivalence tags for marking duplicates or similar/alternative versions within a collection
    pub const FACET_EQUIV: &'static str = "equiv";
}

///////////////////////////////////////////////////////////////////////
/// RefOrigin
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum RefOrigin {
    Track = 1,
    TrackActor = 2,
    Album = 3,
    AlbumActor = 4,
    Release = 5,
}

///////////////////////////////////////////////////////////////////////
/// TrackLock
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum TrackLock {
    Loudness,
    Tempo,
    KeySig,
    TimeSig,
}

impl TrackLock {
    pub fn is_valid(&self) -> bool {
        true
    }
}

///////////////////////////////////////////////////////////////////////
/// Track
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Track {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub resources: Vec<TrackResource>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<ReleaseMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<AlbumMetadata>,

    #[serde(skip_serializing_if = "IndexCount::is_empty", default)]
    pub track_numbers: IndexCount,

    #[serde(skip_serializing_if = "IndexCount::is_empty", default)]
    pub disc_numbers: IndexCount,

    #[serde(skip_serializing_if = "IndexCount::is_empty", default)]
    pub movement_numbers: IndexCount,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub titles: Vec<Title>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub actors: Vec<Actor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<Lyrics>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<SongProfile>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub markers: Vec<TrackMarker>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub locks: Vec<TrackLock>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<ScoredTag>, // no duplicate terms per facet allowed

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub comments: Vec<Comment>, // no duplicate owners allowed

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ratings: Vec<Rating>, // no duplicate owners allowed

    #[serde(rename = "xrefs", skip_serializing_if = "Vec::is_empty", default)]
    pub external_references: Vec<String>,
}

impl Track {
    pub fn is_valid(&self) -> bool {
        !self.resources.is_empty()
            && self.resources.iter().all(TrackResource::is_valid)
            && self.album.iter().all(AlbumMetadata::is_valid)
            && self.release.iter().all(ReleaseMetadata::is_valid)
            && self.track_numbers.is_valid()
            && self.disc_numbers.is_valid()
            && Titles::is_valid(&self.titles)
            && Actors::is_valid(&self.actors)
            && self.lyrics.iter().all(Lyrics::is_valid)
            && self.profile.iter().all(SongProfile::is_valid)
            && self.markers.iter().all(|marker| {
                marker.is_valid()
                    && (!TrackMarker::is_singular(marker.mark)
                        || self.markers
                            .iter()
                            .filter(|marker2| marker.mark == marker2.mark)
                            .count() <= 1)
            }) && self.locks.iter().all(TrackLock::is_valid)
            && self.tags.iter().all(ScoredTag::is_valid)
            && self.ratings.iter().all(Rating::is_valid)
            && self.comments.iter().all(Comment::is_valid)
    }

    pub fn resource<'a>(&'a self, collection_uid: &EntityUid) -> Option<&'a TrackResource> {
        debug_assert!(
            self.resources
                .iter()
                .filter(|resource| &resource.collection.uid == collection_uid)
                .count() <= 1
        );
        self.resources
            .iter()
            .filter(|resource| &resource.collection.uid == collection_uid)
            .nth(0)
    }

    pub fn has_collection(&self, collection_uid: &EntityUid) -> bool {
        self.resource(collection_uid).is_some()
    }

    pub fn main_actor<'a>(&'a self, role: ActorRole) -> Option<&'a Actor> {
        Actors::main_actor(&self.actors, role)
    }

    pub fn album_main_actor<'a>(&'a self, role: ActorRole) -> Option<&'a Actor> {
        self.album
            .as_ref()
            .and_then(|album| Actors::main_actor(&album.actors, role))
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackEntity
///////////////////////////////////////////////////////////////////////

pub type TrackEntity = Entity<Track>;
