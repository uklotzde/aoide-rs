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

use crate::{
    io::import::{self, *},
    util::{
        digest::MediaDigest, parse_artwork_from_embedded_image, parse_key_signature,
        parse_replay_gain, parse_tempo_bpm, parse_year_tag, push_next_actor_role_name,
        tag::import_faceted_tags,
    },
    Result,
};

use aoide_core::{
    audio::{
        channel::{ChannelCount, ChannelLayout, Channels},
        signal::{BitrateBps, BitsPerSecond, SampleRateHz},
        AudioContent,
        PositionMs,
    },
    media::{Content, ContentMetadataFlags},
    music::time::{Beats, TempoBpm},
    tag::{Score as TagScore, Tags, TagsMap},
    track::{
        actor::ActorRole,
        album::AlbumKind,
        tag::{FACET_CGROUP, FACET_COMMENT, FACET_GENRE, FACET_MOOD},
        title::{Title, TitleKind},
        cue::{Cue, CueFlags},
        Track,
    },
    util::{Canonical, CanonicalizeInto as _},
};

use aoide_core_serde::tag::Tags as SerdeTags;

use image::ImageFormat;
use mp4ameta::{
    ChannelConfig, Data, FourCC, FreeformIdent, SampleRate as Mp4SampleRate, Tag as Mp4Tag,
    STANDARD_GENRES,
};
use semval::IsValid as _;
use std::time::Duration;
use triseratops::tag::{ TagContainer as SeratoTagContainer, TagFormat as SeratoTagFormat };

#[derive(Debug)]
pub struct ImportTrack;

fn read_sample_rate(sample_rate: Mp4SampleRate) -> SampleRateHz {
    use Mp4SampleRate::*;
    SampleRateHz(match sample_rate {
        F96000 => 96_000.0,
        F88200 => 88_200.0,
        F64000 => 64_000.0,
        F48000 => 48_000.0,
        F44100 => 44_100.0,
        F32000 => 32_000.0,
        F24000 => 24_000.0,
        F22050 => 22_500.0,
        F16000 => 16_000.0,
        F12000 => 12_000.0,
        F11025 => 11_025.0,
        F8000 => 8_000.0,
    })
}

fn read_bitrate(bitrate: u32) -> Option<BitrateBps> {
    let bits_per_second = bitrate as BitsPerSecond;
    let bitrate_bps = BitrateBps(bits_per_second);
    if bitrate_bps >= BitrateBps::min() {
        Some(bitrate_bps)
    } else {
        None
    }
}

fn read_channels(channel_config: ChannelConfig) -> Channels {
    use ChannelConfig::*;
    match channel_config {
        Mono => Channels::Layout(ChannelLayout::Mono),
        Stereo => Channels::Layout(ChannelLayout::Stereo),
        Three => Channels::Count(ChannelCount(3)),
        Four => Channels::Count(ChannelCount(4)),
        Five => Channels::Count(ChannelCount(5)),
        FiveOne => Channels::Layout(ChannelLayout::FiveOne),
        SevenOne => Channels::Layout(ChannelLayout::SevenOne),
    }
}

const COM_APPLE_ITUNES_FREEFORM_MEAN: &str = "com.apple.iTunes";
const ORG_MIXXX_DJ_FREEFORM_MEAN: &str = "org.mixxx.dj";

impl import::ImportTrack for ImportTrack {
    fn import_track(
        &self,
        config: &ImportTrackConfig,
        options: ImportTrackOptions,
        mut track: Track,
        reader: &mut Box<dyn Reader>,
    ) -> Result<Track> {
        // Extract metadata with mp4ameta
        let mut mp4_tag = Mp4Tag::read_from(reader).map_err(anyhow::Error::from)?;

        if track
            .media_source
            .content_metadata_flags
            .update(ContentMetadataFlags::UNRELIABLE)
        {
            let duration = Some(
                Duration::from_secs_f64(mp4_tag.duration().map_err(anyhow::Error::from)?).into(),
            );
            let channels = Some(
                mp4_tag
                    .channel_config()
                    .map(read_channels)
                    .map_err(anyhow::Error::from)?,
            );
            let sample_rate = Some(
                mp4_tag
                    .sample_rate()
                    .map(read_sample_rate)
                    .map_err(anyhow::Error::from)?,
            );
            let bitrate = read_bitrate(mp4_tag.average_bitrate().map_err(anyhow::Error::from)?);
            let loudness = mp4_tag
                .string(&FreeformIdent::new(
                    COM_APPLE_ITUNES_FREEFORM_MEAN,
                    "replaygain_track_gain",
                ))
                .next()
                .and_then(parse_replay_gain);
            let encoder = mp4_tag.take_encoder();
            let audio_content = AudioContent {
                duration,
                channels,
                sample_rate,
                bitrate,
                loudness,
                encoder,
            };
            track.media_source.content = Content::Audio(audio_content);
        }

        let tempo_bpm = mp4_tag
            .string(&FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "BPM"))
            .flat_map(parse_tempo_bpm)
            .next()
            .or_else(|| mp4_tag.bpm().map(|bpm| TempoBpm(Beats::from(bpm))));
        if let Some(tempo_bpm) = tempo_bpm {
            debug_assert!(tempo_bpm.is_valid());
            track.metrics.tempo_bpm = Some(tempo_bpm);
        }

        let key_signature = mp4_tag
            .string(&FreeformIdent::new(
                COM_APPLE_ITUNES_FREEFORM_MEAN,
                "initialkey",
            ))
            // alternative name (conforms to Rapid Evolution)
            .chain(mp4_tag.string(&FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "KEY")))
            .flat_map(parse_key_signature)
            .next();
        if let Some(key_signature) = key_signature {
            track.metrics.key_signature = key_signature;
        }

        // Track titles
        let mut track_titles = Vec::with_capacity(4);
        if let Some(name) = mp4_tag.take_title() {
            let title = Title {
                name,
                kind: TitleKind::Main,
            };
            track_titles.push(title);
        }
        if let Some(name) = mp4_tag.take_work() {
            let title = Title {
                name,
                kind: TitleKind::Work,
            };
            track_titles.push(title);
        }
        if let Some(name) = mp4_tag.take_movement() {
            let title = Title {
                name,
                kind: TitleKind::Movement,
            };
            track_titles.push(title);
        }
        if let Some(name) = mp4_tag
            .take_string(&FreeformIdent::new(
                COM_APPLE_ITUNES_FREEFORM_MEAN,
                "SUBTITLE",
            ))
            .next()
        {
            let title = Title {
                name,
                kind: TitleKind::Sub,
            };
            track_titles.push(title);
        }
        let track_titles = track_titles.canonicalize_into();
        if !track_titles.is_empty() {
            track.titles = Canonical::tie(track_titles.canonicalize_into());
        }

        // Track actors
        let mut track_actors = Vec::with_capacity(8);
        for name in mp4_tag.take_artists() {
            push_next_actor_role_name(&mut track_actors, ActorRole::Artist, name);
        }
        for name in mp4_tag.take_composers() {
            push_next_actor_role_name(&mut track_actors, ActorRole::Composer, name);
        }
        for name in mp4_tag.take_string(&FreeformIdent::new(
            COM_APPLE_ITUNES_FREEFORM_MEAN,
            "REMIXER",
        )) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Remixer, name);
        }
        for name in mp4_tag.take_string(&FreeformIdent::new(
            COM_APPLE_ITUNES_FREEFORM_MEAN,
            "LYRICIST",
        )) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Lyricist, name);
        }
        for name in mp4_tag.take_string(&FreeformIdent::new(
            COM_APPLE_ITUNES_FREEFORM_MEAN,
            "CONDUCTOR",
        )) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Conductor, name);
        }
        let track_actors = track_actors.canonicalize_into();
        if !track_actors.is_empty() {
            track.actors = Canonical::tie(track_actors);
        }

        let mut album = track.album.untie();

        // Album titles
        let mut album_titles = Vec::with_capacity(1);
        if let Some(name) = mp4_tag.take_album() {
            let title = Title {
                name,
                kind: TitleKind::Main,
            };
            album_titles.push(title);
        }
        let album_titles = album_titles.canonicalize_into();
        if !album_titles.is_empty() {
            album.titles = Canonical::tie(album_titles);
        }

        // Album actors
        let mut album_actors = Vec::with_capacity(4);
        for name in mp4_tag.take_album_artists() {
            push_next_actor_role_name(&mut album_actors, ActorRole::Artist, name);
        }
        let album_actors = album_actors.canonicalize_into();
        if !album_actors.is_empty() {
            album.actors = Canonical::tie(album_actors);
        }

        // Album properties
        if mp4_tag.compilation() {
            album.kind = AlbumKind::Compilation;
        }

        track.album = Canonical::tie(album);

        // Release properties
        if let Some(year) = mp4_tag.year() {
            if let Some(released_at) = parse_year_tag(year) {
                track.release.released_at = Some(released_at);
            }
        }
        if let Some(copyright) = mp4_tag.take_copyright() {
            track.release.copyright = Some(copyright);
        }
        if let Some(label) = mp4_tag
            .take_string(&FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "LABEL"))
            .next()
        {
            track.release.released_by = Some(label);
        }

        let mut tags_map = TagsMap::default();

        // Serato Tags
        if options.contains(ImportTrackOptions::SERATO_TAGS) {
            let mut serato_tags = SeratoTagContainer::new();

            if let Some(data) = mp4_tag
                .data(&FreeformIdent::new(
                    "com.serato.dj",
                    "markers",
                ))
                .next()
            {
                match data {
                    Data::Utf8(input) => {
                        serato_tags.parse_markers(input.as_bytes(), SeratoTagFormat::MP4)
                        .map_err(|err| {
                            log::warn!("Failed to parse Serato Markers: {}", err);
                        }).ok();
                    }
                    data => {
                        log::warn!("Unexpected data for Serato Markers: {:?}", data);
                    }
                }
            }

            if let Some(data) = mp4_tag
                .data(&FreeformIdent::new(
                    "com.serato.dj",
                    "markers2",
                ))
                .next()
            {
                match data {
                    Data::Utf8(input) => {
                        serato_tags.parse_markers2(input.as_bytes(), SeratoTagFormat::MP4)
                        .map_err(|err| {
                            log::warn!("Failed to parse Serato Markers2: {}", err);
                        }).ok();
                    }
                    data => {
                        log::warn!("Unexpected data for Serato Markers2: {:?}", data);
                    }
                }
            }

            let mut track_cues = vec![];

            for serato_cue in serato_tags.cues() {
                let cue = Cue {
                    bank_index: 0,
                    slot_index: Some(serato_cue.index.into()),
                    in_position: Some(PositionMs(serato_cue.position_millis.into())),
                    out_position: None,
                    out_mode: None,
                    label: Some(serato_cue.label),
                    color: None, // TODO
                    flags: CueFlags::empty(),
                };
                track_cues.push(cue);
            }

            for serato_loop in serato_tags.loops() {
                let flags = if serato_loop.is_locked { CueFlags::LOCKED } else { CueFlags::empty() };
                let cue = Cue {
                    bank_index: 1,
                    slot_index: Some(serato_loop.index.into()),
                    in_position: Some(PositionMs(serato_loop.start_position_millis.into())),
                    out_position: Some(PositionMs(serato_loop.start_position_millis.into())),
                    out_mode: None,
                    label: Some(serato_loop.label),
                    color: None, // TODO
                    flags,
                };
                track_cues.push(cue);
            }

            let track_cues = track_cues.canonicalize_into();
            if !track_cues.is_empty() {
                track.cues = Canonical::tie(track_cues);
            }
        }

        // Mixxx CustomTags
        if options.contains(ImportTrackOptions::MIXXX_CUSTOM_TAGS) {
            if let Some(data) = mp4_tag
                .data(&FreeformIdent::new(
                    ORG_MIXXX_DJ_FREEFORM_MEAN,
                    "CustomTags",
                ))
                .next()
            {
                if let Some(custom_tags) = match data {
                    Data::Utf8(input) => serde_json::from_str::<SerdeTags>(input)
                        .map_err(|err| {
                            log::warn!("Failed to parse Mixxx custom tags: {}", err);
                            err
                        })
                        .ok(),
                    data => {
                        log::warn!("Unexpected data for Mixxx custom tags: {:?}", data);
                        None
                    }
                }
                .map(Tags::from)
                {
                    // Initialize map with all existing custom tags as starting point
                    tags_map = custom_tags.into();
                }
            }
        }

        // Comment tag
        if let Some(comment) = mp4_tag.take_comment() {
            tags_map.remove_faceted_tags(&FACET_COMMENT);
            let mut next_score_value = TagScore::default_value();
            import_faceted_tags(
                &mut tags_map,
                &mut next_score_value,
                &FACET_COMMENT,
                None,
                comment,
            );
        }

        // Genre tags
        let mut genre_count = 0;
        if mp4_tag.custom_genres().next().is_some() {
            tags_map.remove_faceted_tags(&FACET_GENRE);
            let tag_mapping_config = config.faceted_tag_mapping.get(FACET_GENRE.value());
            let mut next_score_value = TagScore::max_value();
            for genre in mp4_tag.take_custom_genres() {
                genre_count += import_faceted_tags(
                    &mut tags_map,
                    &mut next_score_value,
                    &FACET_GENRE,
                    tag_mapping_config,
                    genre,
                );
            }
        }
        if genre_count == 0 && mp4_tag.standard_genres().next().is_some() {
            // Import legacy/standard genres instead
            tags_map.remove_faceted_tags(&FACET_GENRE);
            let mut next_score_value = TagScore::max_value();
            for genre_id in mp4_tag.standard_genres() {
                let genre = STANDARD_GENRES
                    .iter()
                    .filter_map(|(id, genre)| if *id == genre_id { Some(*genre) } else { None })
                    .next();
                if let Some(genre) = genre {
                    genre_count += import_faceted_tags(
                        &mut tags_map,
                        &mut next_score_value,
                        &FACET_GENRE,
                        None,
                        genre,
                    );
                }
            }
        }

        // Mood tags
        let mood_ident = FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "MOOD");
        if mp4_tag.string(&mood_ident).next().is_some() {
            tags_map.remove_faceted_tags(&FACET_MOOD);
            let tag_mapping_config = config.faceted_tag_mapping.get(FACET_MOOD.value());
            let mut next_score_value = TagScore::max_value();
            for mood in mp4_tag.take_string(&mood_ident) {
                import_faceted_tags(
                    &mut tags_map,
                    &mut next_score_value,
                    &FACET_MOOD,
                    tag_mapping_config,
                    mood,
                );
            }
        }

        // Grouping tags
        if mp4_tag.groupings().next().is_some() {
            tags_map.remove_faceted_tags(&FACET_CGROUP);
            let tag_mapping_config = config.faceted_tag_mapping.get(FACET_CGROUP.value());
            let mut next_score_value = TagScore::max_value();
            for grouping in mp4_tag.take_groupings() {
                import_faceted_tags(
                    &mut tags_map,
                    &mut next_score_value,
                    &FACET_CGROUP,
                    tag_mapping_config,
                    grouping,
                );
            }
        }

        debug_assert!(track.tags.is_empty());
        track.tags = Canonical::tie(tags_map.into());

        // Indexes (in pairs)
        if mp4_tag.track_number().is_some() || mp4_tag.total_tracks().is_some() {
            track.indexes.track.number = mp4_tag.track_number();
            track.indexes.track.total = mp4_tag.total_tracks();
        }
        if mp4_tag.disc_number().is_some() || mp4_tag.total_discs().is_some() {
            track.indexes.disc.number = mp4_tag.disc_number();
            track.indexes.disc.total = mp4_tag.total_discs();
        }
        if mp4_tag.movement_index().is_some() || mp4_tag.movement_count().is_some() {
            track.indexes.movement.number = mp4_tag.movement_index();
            track.indexes.movement.total = mp4_tag.movement_count();
        }

        // Artwork
        if options.contains(ImportTrackOptions::ARTWORK) {
            let mut image_digest = if options.contains(ImportTrackOptions::ARTWORK_DIGEST) {
                if options.contains(ImportTrackOptions::ARTWORK_DIGEST_SHA256) {
                    // Compatibility
                    MediaDigest::sha256()
                } else {
                    // Default
                    MediaDigest::new()
                }
            } else {
                Default::default()
            };
            for image_data in mp4_tag.data(&FourCC(*b"covr")) {
                let (image_data, image_format) = match image_data {
                    Data::Jpeg(bytes) => (bytes, Some(ImageFormat::Jpeg)),
                    Data::Png(bytes) => (bytes, Some(ImageFormat::Png)),
                    Data::Reserved(bytes) => (bytes, None),
                    _ => {
                        log::warn!("Unexpected cover art data");
                        break;
                    }
                };
                if let Some(artwork) =
                    parse_artwork_from_embedded_image(image_data, image_format, &mut image_digest)
                {
                    track.media_source.artwork = artwork;
                    break;
                }
            }
        }

        Ok(track)
    }
}
