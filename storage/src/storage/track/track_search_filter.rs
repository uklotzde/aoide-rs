use super::*;

// TODO: How can we remove this ugly type alias definition?
type TrackSearchBoxedQuery<'a> = diesel::query_builder::BoxedSelectStatement<
    'a,
    (
        diesel::sql_types::BigInt,
        diesel::sql_types::Binary,
        diesel::sql_types::BigInt,
        diesel::sql_types::Timestamp,
        diesel::sql_types::SmallInt,
        diesel::sql_types::Integer,
        diesel::sql_types::Integer,
        diesel::sql_types::Binary,
    ),
    diesel::query_source::joins::JoinOn<
        diesel::query_source::joins::Join<
            diesel::query_source::joins::JoinOn<
                diesel::query_source::joins::Join<
                    diesel::query_source::joins::JoinOn<
                        diesel::query_source::joins::Join<
                            diesel::query_source::joins::JoinOn<
                                diesel::query_source::joins::Join<
                                    tbl_track::table,
                                    aux_track_overview::table,
                                    diesel::query_source::joins::Inner,
                                >,
                                diesel::expression::operators::Eq<
                                    diesel::expression::nullable::Nullable<
                                        aux_track_overview::columns::track_id,
                                    >,
                                    diesel::expression::nullable::Nullable<tbl_track::columns::id>,
                                >,
                            >,
                            aux_track_summary::table,
                            diesel::query_source::joins::Inner,
                        >,
                        diesel::expression::operators::Eq<
                            diesel::expression::nullable::Nullable<
                                aux_track_summary::columns::track_id,
                            >,
                            diesel::expression::nullable::Nullable<tbl_track::columns::id>,
                        >,
                    >,
                    aux_track_source::table,
                    diesel::query_source::joins::LeftOuter,
                >,
                diesel::expression::operators::Eq<
                    diesel::expression::nullable::Nullable<aux_track_source::columns::track_id>,
                    diesel::expression::nullable::Nullable<tbl_track::columns::id>,
                >,
            >,
            aux_track_collection::table,
            diesel::query_source::joins::LeftOuter,
        >,
        diesel::expression::operators::Eq<
            diesel::expression::nullable::Nullable<aux_track_collection::columns::track_id>,
            diesel::expression::nullable::Nullable<tbl_track::columns::id>,
        >,
    >,
    diesel::sqlite::Sqlite,
>;

pub trait TrackSearchFilter {
    fn apply_to_query<'a>(
        &'a self,
        query: TrackSearchBoxedQuery<'a>,
        collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedQuery<'a>;
}

impl TrackSearchFilter for PhraseFilter {
    fn apply_to_query<'a>(
        &'a self,
        mut query: TrackSearchBoxedQuery<'a>,
        _: Option<&EntityUid>,
    ) -> TrackSearchBoxedQuery<'a> {
        // Escape wildcard character with backslash (see below)
        let escaped = self.phrase.replace('\\', "\\\\").replace('%', "\\%");
        let escaped_and_tokenized = escaped.split_whitespace().filter(|token| !token.is_empty());
        let escaped_and_tokenized_len = escaped_and_tokenized
            .clone()
            .fold(0, |len, token| len + token.len());
        // TODO: Use Rc<String> to avoid cloning strings?
        let like_expr = if escaped_and_tokenized_len > 0 {
            let mut like_expr = escaped_and_tokenized.fold(
                String::with_capacity(1 + escaped_and_tokenized_len + 1), // leading/trailing '%'
                |mut like_expr, part| {
                    // Prepend wildcard character before each part
                    like_expr.push('%');
                    like_expr.push_str(part);
                    like_expr
                },
            );
            // Append final wildcard character after last part
            like_expr.push('%');
            like_expr
        } else {
            // unused
            String::new()
        };

        if !like_expr.is_empty() {
            // aux_track_source (join)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::SourceUri)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_source::content_uri_decoded
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_source::content_uri_decoded
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::SourceType)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_source::content_type
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_source::content_type
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }

            // aux_track_overview (join)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::TrackTitle)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_overview::track_title
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_overview::track_title
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::AlbumTitle)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_overview::album_title
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_overview::album_title
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }

            // aux_track_summary (join)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::TrackArtist)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_summary::track_artist
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_summary::track_artist
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::AlbumArtist)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_summary::album_artist
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_summary::album_artist
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }

            // aux_track_comment (subselect)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::Comments)
            {
                let subselect = aux_track_comment::table
                    .select(aux_track_comment::track_id)
                    .filter(aux_track_comment::text.like(like_expr.clone()).escape('\\'));
                query = match self.modifier {
                    None => query.or_filter(tbl_track::id.eq_any(subselect)),
                    Some(FilterModifier::Complement) => {
                        query.or_filter(tbl_track::id.ne_all(subselect))
                    }
                };
            }
        }
        query
    }
}

impl TrackSearchFilter for NumericFilter {
    fn apply_to_query<'a>(
        &'a self,
        query: TrackSearchBoxedQuery<'a>,
        _: Option<&EntityUid>,
    ) -> TrackSearchBoxedQuery<'a> {
        match select_track_ids_from_profile_matching_numeric_filter(self) {
            Some((subselect, filter_modifier)) => match filter_modifier {
                None => query.filter(tbl_track::id.eq_any(subselect)),
                Some(FilterModifier::Complement) => query.filter(tbl_track::id.ne_all(subselect)),
            },
            None => match self.field {
                NumericField::DurationMs => match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_duration_ms.lt(self.condition.value),
                            ),
                            Some(FilterModifier::Complement) => query.filter(not(
                                aux_track_source::audio_duration_ms.lt(self.condition.value),
                            )),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_duration_ms.ge(self.condition.value),
                            ),
                            Some(FilterModifier::Complement) => query.filter(not(
                                aux_track_source::audio_duration_ms.ge(self.condition.value),
                            )),
                        },
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_duration_ms.gt(self.condition.value),
                            ),
                            Some(FilterModifier::Complement) => query.filter(not(
                                aux_track_source::audio_duration_ms.gt(self.condition.value),
                            )),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_duration_ms.le(self.condition.value),
                            ),
                            Some(FilterModifier::Complement) => query.filter(not(
                                aux_track_source::audio_duration_ms.le(self.condition.value),
                            )),
                        },
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_duration_ms.eq(self.condition.value),
                            ),
                            Some(FilterModifier::Complement) => query.filter(not(
                                aux_track_source::audio_duration_ms.eq(self.condition.value),
                            )),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_duration_ms.ne(self.condition.value),
                            ),
                            Some(FilterModifier::Complement) => query.filter(not(
                                aux_track_source::audio_duration_ms.ne(self.condition.value),
                            )),
                        },
                    },
                },
                NumericField::SampleRateHz => match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_samplerate_hz
                                    .lt(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_samplerate_hz
                                    .lt(self.condition.value as i32))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_samplerate_hz
                                    .ge(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_samplerate_hz
                                    .ge(self.condition.value as i32))),
                        },
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_samplerate_hz
                                    .gt(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_samplerate_hz
                                    .gt(self.condition.value as i32))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_samplerate_hz
                                    .le(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_samplerate_hz
                                    .le(self.condition.value as i32))),
                        },
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_samplerate_hz
                                    .eq(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_samplerate_hz
                                    .eq(self.condition.value as i32))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_samplerate_hz
                                    .ne(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_samplerate_hz
                                    .ne(self.condition.value as i32))),
                        },
                    },
                },
                NumericField::BitRateBps => match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_bitrate_bps.lt(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_bitrate_bps
                                    .lt(self.condition.value as i32))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_bitrate_bps.ge(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_bitrate_bps
                                    .ge(self.condition.value as i32))),
                        },
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_bitrate_bps.gt(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_bitrate_bps
                                    .gt(self.condition.value as i32))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_bitrate_bps.le(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_bitrate_bps
                                    .le(self.condition.value as i32))),
                        },
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_bitrate_bps.eq(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_bitrate_bps
                                    .eq(self.condition.value as i32))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_bitrate_bps.ne(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_bitrate_bps
                                    .ne(self.condition.value as i32))),
                        },
                    },
                },
                NumericField::ChannelsCount => match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_channels_count
                                    .lt(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_channels_count
                                    .lt(self.condition.value as i16))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_channels_count
                                    .ge(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_channels_count
                                    .ge(self.condition.value as i16))),
                        },
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_channels_count
                                    .gt(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_channels_count
                                    .gt(self.condition.value as i16))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_channels_count
                                    .le(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_channels_count
                                    .le(self.condition.value as i16))),
                        },
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_channels_count
                                    .eq(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_channels_count
                                    .eq(self.condition.value as i16))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_channels_count
                                    .ne(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_channels_count
                                    .ne(self.condition.value as i16))),
                        },
                    },
                },
                numeric_field => {
                    unreachable!("unhandled numeric filter field: {:?}", numeric_field)
                }
            },
        }
    }
}

impl TrackSearchFilter for TagFilter {
    fn apply_to_query<'a>(
        &'a self,
        query: TrackSearchBoxedQuery<'a>,
        _: Option<&EntityUid>,
    ) -> TrackSearchBoxedQuery<'a> {
        let (subselect, filter_modifier) = select_track_ids_matching_tag_filter(&self);
        match filter_modifier {
            None => query.filter(tbl_track::id.eq_any(subselect)),
            Some(FilterModifier::Complement) => query.filter(tbl_track::id.ne_all(subselect)),
        }
    }
}

impl TrackSearchFilter for TrackSort {
    fn apply_to_query<'a>(
        &'a self,
        query: TrackSearchBoxedQuery<'a>,
        collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedQuery<'a> {
        let direction = self
            .direction
            .unwrap_or_else(|| TrackSort::default_direction(self.field));
        match self.field {
            field @ TrackSortField::InCollectionSince => {
                if collection_uid.is_some() {
                    match direction {
                        SortDirection::Ascending => {
                            query.then_order_by(aux_track_collection::since.asc())
                        }
                        SortDirection::Descending => {
                            query.then_order_by(aux_track_collection::since.desc())
                        }
                    }
                } else {
                    warn!("Cannot order by {:?} over multiple collections", field);
                    query
                }
            }
            TrackSortField::LastRevisionedAt => match direction {
                SortDirection::Ascending => query.then_order_by(tbl_track::rev_timestamp.asc()),
                SortDirection::Descending => query.then_order_by(tbl_track::rev_timestamp.desc()),
            },
            TrackSortField::TrackTitle => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_overview::track_title.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_overview::track_title.desc())
                }
            },
            TrackSortField::AlbumTitle => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_overview::album_title.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_overview::album_title.desc())
                }
            },
            TrackSortField::ReleasedAt => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_overview::released_at.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_overview::released_at.desc())
                }
            },
            TrackSortField::ReleasedBy => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_overview::released_by.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_overview::released_by.desc())
                }
            },
            TrackSortField::TrackArtist => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_summary::track_artist.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_summary::track_artist.desc())
                }
            },
            TrackSortField::AlbumArtist => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_summary::album_artist.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_summary::album_artist.desc())
                }
            },
        }
    }
}