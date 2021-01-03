-- aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
--
-- This program is free software: you can redistribute it and/or modify
-- it under the terms of the GNU Affero General Public License as
-- published by the Free Software Foundation, either version 3 of the
-- License, or (at your option) any later version.
--
-- This program is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY; without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU Affero General Public License for more details.
--
-- You should have received a copy of the GNU Affero General Public License
-- along with this program.  If not, see <https://www.gnu.org/licenses/>.

CREATE TABLE IF NOT EXISTS track_actor (
    row_id                   INTEGER PRIMARY KEY,
    -- relations (immutable)
    track_id                 INTEGER NOT NULL,
    -- properties
    scope                    TINYINT NOT NULL, -- 0: track, 1: album
    kind                     TINYINT NOT NULL,
    name                     TEXT NOT NULL,
    role                     TINYINT NOT NULL,
    role_notes               TEXT,
    --
    FOREIGN KEY(track_id) REFERENCES track(row_id)
);

CREATE INDEX IF NOT EXISTS idx_track_actor_track_id ON track_actor (
    track_id
);

CREATE INDEX IF NOT EXISTS idx_track_actor_name_scope_kind_role ON track_actor (
    name,
    scope,
    kind,
    role
);
