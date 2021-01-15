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

///////////////////////////////////////////////////////////////////////

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

use aoide_core::{
    media::{Content, ContentMetadataStatus, Source},
    tag::{
        Facet as TagFacet, FacetValue, Label as TagLabel, LabelValue, PlainTag, Score as TagScore,
        ScoreValue, TagsMap,
    },
    track::{
        actor::{Actor, ActorKind, ActorRole},
        Track,
    },
    util::clock::DateTime,
};

use anyhow::anyhow;
use bitflags::bitflags;
use semval::IsValid;
use std::{
    collections::HashMap,
    fs::File,
    io::{Error as IoError, ErrorKind, Read, Seek},
    ops::{Deref, DerefMut},
    result::Result as StdResult,
};
use thiserror::Error;
use url::Url;

pub use mime::Mime;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error("unknown content type")]
    UnknownContentType,

    #[error("unsupported content type")]
    UnsupportedContentType,

    #[error("unsupported import options")]
    UnsupportedImportOptions,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = StdResult<T, Error>;

#[rustfmt::skip]
bitflags! {
    pub struct ImportTrackOptions: u16 {
        const METADATA              = 0b0000000000000001;
        const ARTWORK               = 0b0000000000000010;
        const CONTENT_DIGEST_SHA256 = 0b0000000000000100;
        const ARTWORK_DIGEST_SHA256 = 0b0000000000001000;
        // Custom application metadata
        const MIXXX_CUSTOM_TAGS     = 0b0000000100000000;
        const SERATO_MARKERS        = 0b0000001000000000;
    }
}

impl ImportTrackOptions {
    pub fn is_valid(self) -> bool {
        Self::all().contains(self)
    }
}

impl Default for ImportTrackOptions {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImportTrackConfig {
    pub faceted_tag_mapping: FacetedTagMappingConfig,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ImportTrackInput {
    pub collected_at: DateTime,
    pub synchronized_at: DateTime,
}

pub fn guess_mime_from_url(url: &Url) -> Result<Mime> {
    let mime_guess = mime_guess::from_path(url.path());
    if mime_guess.first().is_none() {
        return Err(Error::UnknownContentType);
    }
    mime_guess
        .into_iter()
        .find(|mime| mime.type_() == mime::AUDIO)
        .ok_or(Error::UnsupportedContentType)
}

impl ImportTrackInput {
    pub fn try_from_url_into_new_track(self, url: &Url, mime: &Mime) -> Result<Track> {
        let Self {
            collected_at,
            synchronized_at,
        } = self;
        let media_source = Source {
            collected_at,
            synchronized_at: Some(synchronized_at),
            uri: url.to_string(),
            content_type: mime.to_string(),
            content_digest: None,
            content_metadata_status: ContentMetadataStatus::Unknown,
            content: Content::Audio(Default::default()),
            artwork: Default::default(),
        };
        Ok(Track::new_from_media_source(media_source))
    }
}

pub fn import_track_default(
    url: &Url,
    mime: &Mime,
    input: ImportTrackInput,
    options: ImportTrackOptions,
) -> Result<Track> {
    if !options.is_empty() {
        return Err(Error::UnsupportedImportOptions);
    }
    input.try_from_url_into_new_track(url, mime)
}

pub trait Reader: Read + Seek + 'static {}

impl<T> Reader for T where T: Read + Seek + 'static {}

pub trait ImportTrack {
    fn import_track(
        &self,
        url: &Url,
        mime: &Mime,
        _config: &ImportTrackConfig,
        options: ImportTrackOptions,
        input: ImportTrackInput,
        _reader: &mut Box<dyn Reader>,
        _size: u64,
    ) -> Result<Track> {
        import_track_default(url, mime, input, options)
    }
}

pub fn open_local_file_url_for_reading(url: &Url) -> Result<File> {
    log::debug!("Opening local file URL '{}' for reading", url);
    if url.scheme() != "file" {
        return Err(Error::Io(IoError::new(
            ErrorKind::Other,
            anyhow!("Unsupported URL scheme '{}'", url.scheme()),
        )));
    }
    if let Ok(file_path) = url.to_file_path() {
        log::debug!("Importing track from local file {:?}", file_path);
        Ok(File::open(std::path::Path::new(&file_path))?)
    } else {
        log::debug!(
            "Failed to convert URL '{}', into a local, absolute file path",
            url
        );
        Err(Error::Io(IoError::new(
            ErrorKind::Other,
            anyhow!("Invalid or unsupported URL: {}", url),
        )))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TagMappingConfig {
    pub label_separator: LabelValue,
    pub split_score_attenuation: ScoreValue,
}

impl TagMappingConfig {
    pub fn next_score_value(&self, score: ScoreValue) -> ScoreValue {
        debug_assert!(self.split_score_attenuation > TagScore::min().into());
        score * self.split_score_attenuation
    }
}

pub type FacetedTagMappingConfigInner = HashMap<FacetValue, TagMappingConfig>;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FacetedTagMappingConfig(FacetedTagMappingConfigInner);

impl FacetedTagMappingConfig {
    pub const fn new(inner: FacetedTagMappingConfigInner) -> Self {
        Self(inner)
    }
}

impl From<FacetedTagMappingConfigInner> for FacetedTagMappingConfig {
    fn from(inner: FacetedTagMappingConfigInner) -> Self {
        Self::new(inner)
    }
}

impl From<FacetedTagMappingConfig> for FacetedTagMappingConfigInner {
    fn from(outer: FacetedTagMappingConfig) -> Self {
        let FacetedTagMappingConfig(inner) = outer;
        inner
    }
}

impl Deref for FacetedTagMappingConfig {
    type Target = FacetedTagMappingConfigInner;

    fn deref(&self) -> &Self::Target {
        let Self(inner) = self;
        inner
    }
}

impl DerefMut for FacetedTagMappingConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self(inner) = self;
        inner
    }
}

fn try_import_plain_tag(
    label_value: impl Into<LabelValue>,
    score_value: impl Into<ScoreValue>,
) -> StdResult<PlainTag, PlainTag> {
    let label = TagLabel::clamp_from(label_value);
    let score = TagScore::clamp_from(score_value);
    let plain_tag = PlainTag {
        label: Some(label),
        score,
    };
    if plain_tag.is_valid() {
        Ok(plain_tag)
    } else {
        Err(plain_tag)
    }
}

fn import_faceted_tags(
    tags_map: &mut TagsMap,
    next_score_value: &mut ScoreValue,
    facet: &TagFacet,
    tag_mapping_config: Option<&TagMappingConfig>,
    label_value: impl Into<LabelValue>,
) -> usize {
    let mut import_count = 0;
    let label_value = label_value.into();
    if let Some(tag_mapping_config) = tag_mapping_config {
        if !tag_mapping_config.label_separator.is_empty() {
            for (_, split_label_value) in
                label_value.match_indices(&tag_mapping_config.label_separator)
            {
                match try_import_plain_tag(split_label_value, *next_score_value) {
                    Ok(plain_tag) => {
                        tags_map.insert(facet.to_owned().into(), plain_tag);
                        import_count += 1;
                        *next_score_value = tag_mapping_config.next_score_value(*next_score_value);
                    }
                    Err(plain_tag) => {
                        log::warn!("Failed to import faceted '{}' tag: {:?}", facet, plain_tag,);
                    }
                }
            }
        }
    }
    if import_count == 0 {
        match try_import_plain_tag(label_value, *next_score_value) {
            Ok(plain_tag) => {
                tags_map.insert(facet.to_owned().into(), plain_tag);
                import_count += 1;
                if let Some(tag_mapping_config) = tag_mapping_config {
                    *next_score_value = tag_mapping_config.next_score_value(*next_score_value);
                }
            }
            Err(plain_tag) => {
                log::warn!("Failed to import faceted '{}' tag: {:?}", facet, plain_tag,);
            }
        }
    }
    import_count
}

fn adjust_last_actor_kind(actors: &mut [Actor], role: ActorRole) -> ActorKind {
    if let Some(last_actor) = actors.last_mut() {
        if last_actor.role == role {
            // ActorKind::Summary is only allowed once for each role
            last_actor.kind = ActorKind::Primary;
            return ActorKind::Primary;
        }
    }
    ActorKind::Summary
}

#[cfg(feature = "feature-flac")]
pub mod flac;

#[cfg(feature = "feature-mp3")]
pub mod mp3;

#[cfg(feature = "feature-mp4")]
pub mod mp4;

#[cfg(feature = "feature-ogg")]
pub mod ogg;

#[cfg(feature = "feature-wav")]
pub mod wav;
