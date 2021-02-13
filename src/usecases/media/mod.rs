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

use aoide_core::{entity::EntityUid, track::Track, util::clock::DateTime};

use aoide_media::{
    fmt::{flac, mp3, mp4, ogg},
    fs::open_local_file_url_for_reading,
    io::import::*,
    util::guess_mime_from_url,
};

use aoide_repo::{collection::EntityRepo as _, media::source::Repo as _};

use std::{io::BufReader, path::PathBuf};
use url::Url;

///////////////////////////////////////////////////////////////////////

pub mod tracker;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ImportMode {
    Once,
    Modified,
    Always,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SynchronizedImportMode {
    Once {
        synchronized_before: bool,
    },
    Modified {
        last_synchronized_at: Option<DateTime>,
    },
    Always,
}

impl SynchronizedImportMode {
    pub const fn new(import_mode: ImportMode, last_synchronized_at: Option<DateTime>) -> Self {
        match import_mode {
            ImportMode::Once => Self::Once {
                synchronized_before: last_synchronized_at.is_some(),
            },
            ImportMode::Modified => Self::Modified {
                last_synchronized_at,
            },
            ImportMode::Always => Self::Always,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportTrackFromFileOutcome {
    Imported(Track),
    SkippedSynchronized(DateTime),
    SkippedDirectory,
}

pub fn import_track_from_url(
    url: &Url,
    mode: SynchronizedImportMode,
    config: &ImportTrackConfig,
    flags: ImportTrackFlags,
    collected_at: DateTime,
) -> Result<ImportTrackFromFileOutcome> {
    let (file_path, file) = if let Some((file_path, file)) = open_local_file_url_for_reading(url)? {
        (file_path, file)
    } else {
        log::debug!("URL {} is a directory", url);
        return Ok(ImportTrackFromFileOutcome::SkippedDirectory);
    };
    let file_metadata = file.metadata().map_err(MediaError::from)?;
    let mime = guess_mime_from_url(url)?;
    let last_modified_at = file_metadata
        .modified()
        .map(DateTime::from)
        .unwrap_or_else(|_| {
            log::error!("Using current time instead of inaccessible last modification time");
            DateTime::now_utc()
        });
    match mode {
        SynchronizedImportMode::Once {
            synchronized_before,
        } => {
            if synchronized_before {
                log::debug!(
                    "Skipping reimport of file {} last modified at {}",
                    file_path.display(),
                    last_modified_at,
                );
                return Ok(ImportTrackFromFileOutcome::SkippedSynchronized(
                    last_modified_at,
                ));
            }
        }
        SynchronizedImportMode::Modified {
            last_synchronized_at,
        } => {
            if let Some(last_synchronized_at) = last_synchronized_at {
                if last_modified_at <= last_synchronized_at {
                    log::debug!(
                        "Skipping reimport of synchronized file {} modified at {} <= {}",
                        file_path.display(),
                        last_modified_at,
                        last_synchronized_at
                    );
                    return Ok(ImportTrackFromFileOutcome::SkippedSynchronized(
                        last_modified_at,
                    ));
                }
            } else {
                log::debug!(
                    "Last synchronization of file {} modified at {} is unknown",
                    file_path.display(),
                    last_modified_at
                );
            }
        }
        SynchronizedImportMode::Always => {
            // Continue regardless of last_modified_at
        }
    }
    let input = NewTrackInput {
        collected_at,
        synchronized_at: last_modified_at,
    };
    let mut reader: Box<dyn Reader> = Box::new(BufReader::new(file));
    let new_track = input.try_from_url_into_new_track(url, &mime)?;
    let track = match mime.as_ref() {
        "audio/flac" => flac::ImportTrack.import_track(config, flags, new_track, &mut reader),
        "audio/mpeg" => mp3::ImportTrack.import_track(config, flags, new_track, &mut reader),
        "audio/m4a" | "video/mp4" => {
            mp4::ImportTrack.import_track(config, flags, new_track, &mut reader)
        }
        "audio/ogg" => ogg::ImportTrack.import_track(config, flags, new_track, &mut reader),
        _ => Err(MediaError::UnsupportedContentType(mime)),
    }?;
    Ok(ImportTrackFromFileOutcome::Imported(track))
}

pub fn relocate_collected_sources(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    old_uri_prefix: &str,
    new_uri_prefix: &str,
) -> Result<usize> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselRepoError, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        let updated_at = DateTime::now_utc();
        Ok(db.relocate_media_sources_by_uri_prefix(
            updated_at,
            collection_id,
            old_uri_prefix,
            new_uri_prefix,
        )?)
    })
    .map_err(Into::into)
}
