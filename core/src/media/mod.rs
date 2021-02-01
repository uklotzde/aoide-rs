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

use std::borrow::Cow;

use crate::{
    audio::{AudioContent, AudioContentInvalidity},
    prelude::*,
};

use bitflags::bitflags;

///////////////////////////////////////////////////////////////////////
// Content
///////////////////////////////////////////////////////////////////////

bitflags! {
    /// A bitmask for controlling how and if content metadata is
    /// re-imported from the source.
    pub struct ContentMetadataFlags: u8 {
        /// Use case: Parsed from file tags which are considered inaccurate
        /// and are often imprecise.
        const UNRELIABLE = 0b00000000;

        /// Use case: Reported by a decoder when opening an audio/video
        /// stream for reading. Nevertheless different decoders may report
        /// slightly differing values.
        const RELIABLE   = 0b00000001;

        /// Locked metadata will not be updated automatically, neither when
        /// parsing file tags nor when decoding an audio/video stream.
        ///
        /// While locked the stale flag is never set.
        const LOCKED     = 0b00000010;

        /// Stale metadata should be re-imported depending on the other
        /// flags.
        const STALE      = 0b00000100;
    }
}

impl ContentMetadataFlags {
    pub fn is_valid(self) -> bool {
        Self::all().contains(self)
    }

    pub fn is_unreliable(self) -> bool {
        !self.intersects(Self::RELIABLE | Self::LOCKED)
    }

    pub fn is_reliable(self) -> bool {
        self.intersects(Self::RELIABLE)
    }

    pub fn is_locked(self) -> bool {
        self.intersects(Self::LOCKED)
    }

    pub fn is_stale(self) -> bool {
        self.intersects(Self::STALE)
    }

    /// Update the current state
    ///
    /// If the given target state is considered at least as reliable
    /// as the current state then modifications are allowed by returning
    /// `true` and the new target state is established.
    ///
    /// Otherwise the current state is preserved. The return value
    /// `false` indicates that modification of metadata is not desired
    /// to prevent loss of accuracy or precision. Instead the stale flag
    /// is set (only if currently not locked) to indicate that an update
    /// from a more reliable source of metadata should be considered.
    ///
    /// The given target state MUST NOT be marked as stale!
    pub fn update(&mut self, target: Self) -> bool {
        debug_assert!(!target.is_stale());
        if (*self - Self::STALE) == target
            || target.is_locked()
            || (!self.is_locked() && target.is_reliable())
        {
            *self = target;
            true
        } else {
            // Metadata does not get stale while locked
            if !self.is_locked() {
                *self |= Self::STALE;
            }
            false
        }
    }
}

impl Default for ContentMetadataFlags {
    fn default() -> Self {
        Self::UNRELIABLE
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct ContentMetadataFlagsInvalidity;

impl Validate for ContentMetadataFlags {
    type Invalidity = ContentMetadataFlagsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !ContentMetadataFlags::is_valid(*self),
                ContentMetadataFlagsInvalidity,
            )
            .into()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    Audio(AudioContent),
}

impl From<AudioContent> for Content {
    fn from(from: AudioContent) -> Self {
        Self::Audio(from)
    }
}

///////////////////////////////////////////////////////////////////////
// Encoder
///////////////////////////////////////////////////////////////////////

/// Concatenate encoder properties
///
/// Some but not all file formats specify two different encoder
/// properties, namely *encoded by* and *encoder settings*. In
/// aoide those properties are represented by a single string.
///
/// Either of the strings might be empty if unknown.
pub fn concat_encoder_strings<'a, T>(encoded_by: &'a T, encoder_settings: &'a T) -> Cow<'a, str>
where
    T: AsRef<str> + ?Sized,
{
    let encoded_by = encoded_by.as_ref().trim();
    let encoder_settings = encoder_settings.as_ref().trim();
    if encoded_by.is_empty() {
        Cow::Borrowed(encoder_settings)
    } else if encoder_settings.is_empty() {
        Cow::Borrowed(encoded_by)
    } else {
        // Concatenate both strings into a single field
        debug_assert!(!encoded_by.is_empty());
        debug_assert!(!encoder_settings.is_empty());
        Cow::Owned(format!("{} {}", encoded_by, encoder_settings))
    }
}

/// Concatenate encoder properties
///
/// Some but not all file formats specify two different encoder
/// properties, namely *encoded by* and *encoder settings*. In
/// aoide those properties are represented by a single string.
///
/// Both properties are optional.
pub fn concat_encoder_properties<'a>(
    encoded_by: Option<&'a str>,
    encoder_settings: Option<&'a str>,
) -> Option<Cow<'a, str>> {
    let encoder = concat_encoder_strings(
        encoded_by.unwrap_or_default(),
        encoder_settings.unwrap_or_default(),
    );
    if encoder.is_empty() {
        None
    } else {
        Some(encoder)
    }
}

///////////////////////////////////////////////////////////////////////
// Artwork
///////////////////////////////////////////////////////////////////////

pub type ImageDimension = u16;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ImageSize {
    pub width: ImageDimension,
    pub height: ImageDimension,
}

impl ImageSize {
    pub const fn is_empty(self) -> bool {
        !(self.width > 0 && self.height > 0)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ImageSizeInvalidity {
    Empty,
}

impl Validate for ImageSize {
    type Invalidity = ImageSizeInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.is_empty(), ImageSizeInvalidity::Empty)
            .into()
    }
}

// All artwork properties are optional for maximum flexibility.
// Properties could be missing or are yet unknown at some point
// in time.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Artwork {
    /// The URI of an external resource
    pub uri: Option<String>,

    /// The media type (if known), e.g. "image/jpeg"
    pub media_type: Option<String>,

    /// Identifies the actual content for cache lookup and to decide
    /// about modifications, e.g. a base64-encoded SHA256 hash of the
    /// raw image data.
    pub digest: Option<Vec<u8>>,

    /// The dimensions of the image (if known).
    pub size: Option<ImageSize>,

    /// An optional (background) color can be used to quickly display
    /// a preliminary view before the actual image has been loaded and
    /// for selecting a matching color scheme.
    pub color_rgb: Option<RgbColor>,
}

impl Artwork {
    pub fn is_empty(&self) -> bool {
        let Self {
            uri,
            media_type,
            digest,
            size,
            color_rgb,
        } = self;
        uri.is_none()
            && media_type.is_none()
            && digest.is_none()
            && size.is_none()
            && color_rgb.is_none()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ArtworkInvalidity {
    MediaTypeEmpty,
    DigestEmpty,
    ImageSize(ImageSizeInvalidity),
    RgbColor(RgbColorInvalidity),
}

impl Validate for Artwork {
    type Invalidity = ArtworkInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                self.media_type
                    .as_ref()
                    .map(String::is_empty)
                    .unwrap_or(false),
                Self::Invalidity::MediaTypeEmpty,
            )
            .invalidate_if(
                self.digest.as_ref().map(Vec::is_empty).unwrap_or(false),
                Self::Invalidity::DigestEmpty,
            )
            .validate_with(&self.size, Self::Invalidity::ImageSize)
            .validate_with(&self.color_rgb, Self::Invalidity::RgbColor)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Source
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub collected_at: DateTime,

    pub synchronized_at: Option<DateTime>,

    pub uri: String,

    pub content_type: String,

    /// Content digest for identifying sources independent of their
    /// URI, e.g. to detect moved files.
    ///
    /// The digest should be calculated from the raw stream data
    /// that is supposed to be read-only and immutable over time.
    /// Additional metadata like file tags that is modified
    /// frequently is not suitable to be included in the digest
    /// calculation.
    pub content_digest: Option<Vec<u8>>,

    pub content_metadata_flags: ContentMetadataFlags,

    pub content: Content,

    pub artwork: Artwork,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SourceInvalidity {
    UriEmpty,
    ContentTypeEmpty,
    ContentMetadataFlags(ContentMetadataFlagsInvalidity),
    AudioContent(AudioContentInvalidity),
    Artwork(ArtworkInvalidity),
}

impl Validate for Source {
    type Invalidity = SourceInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new()
            .invalidate_if(self.uri.trim().is_empty(), Self::Invalidity::UriEmpty)
            .invalidate_if(
                self.content_type.trim().is_empty(),
                Self::Invalidity::ContentTypeEmpty,
            )
            .validate_with(
                &self.content_metadata_flags,
                Self::Invalidity::ContentMetadataFlags,
            )
            .validate_with(&self.artwork, Self::Invalidity::Artwork);
        // TODO: Validate MIME type
        match self.content {
            Content::Audio(ref audio_content) => {
                context.validate_with(audio_content, Self::Invalidity::AudioContent)
            }
        }
        .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceUri {
    pub uri: String,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SourceUriInvalidity {
    UriEmpty,
}

impl Validate for SourceUri {
    type Invalidity = SourceUriInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.uri.trim().is_empty(), Self::Invalidity::UriEmpty)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SourceOrUri {
    Source(Source),
    Uri(SourceUri),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SourceOrUriInvalidity {
    Source(SourceInvalidity),
    Uri(SourceUriInvalidity),
}

impl Validate for SourceOrUri {
    type Invalidity = SourceOrUriInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new();
        use SourceOrUri::*;
        match self {
            Source(source) => context.validate_with(source, Self::Invalidity::Source),
            Uri(source_uri) => context.validate_with(source_uri, Self::Invalidity::Uri),
        }
        .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
