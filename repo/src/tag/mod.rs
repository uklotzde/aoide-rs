// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::tag::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filter {
    pub modifier: Option<FilterModifier>,

    // Facets are always matched with equals. Use an empty vector
    // for matching only tags without a facet.
    pub facets: Option<Vec<String>>,

    pub label: Option<StringPredicate>,

    pub score: Option<NumericPredicate>,
}

impl Filter {
    pub fn any_facet() -> Option<Vec<String>> {
        None
    }

    pub fn no_facet() -> Option<Vec<String>> {
        Some(Vec::default())
    }

    pub fn any_term() -> Option<StringPredicate> {
        None
    }

    pub fn any_score() -> Option<NumericPredicate> {
        None
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SortField {
    Facet,
    Label,
    Score,
    Count,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SortOrder {
    pub field: SortField,
    pub direction: SortDirection,
}

fn dedup_facets(facets: &mut Vec<tag::Facet>) {
    facets.sort_unstable();
    facets.dedup();
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CountParams {
    pub facets: Option<Vec<tag::Facet>>,
    pub include_non_faceted_tags: Option<bool>,
    pub ordering: Vec<tag::SortOrder>,
}

impl CountParams {
    pub fn dedup_facets(&mut self) {
        if let Some(ref mut facets) = self.facets {
            tag::dedup_facets(facets);
        }
    }

    pub fn include_non_faceted_tags(&self) -> bool {
        self.include_non_faceted_tags.unwrap_or(true)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FacetCountParams {
    pub facets: Option<Vec<tag::Facet>>,
    pub ordering: Vec<tag::SortOrder>,
}

impl FacetCountParams {
    pub fn dedup_facets(&mut self) {
        if let Some(ref mut facets) = self.facets {
            tag::dedup_facets(facets);
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct FacetCount {
    pub facet: Facet,
    pub total_count: usize,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct AvgScoreCount {
    pub facet: Option<Facet>,
    pub label: Option<Label>,
    pub avg_score: Score,
    pub total_count: usize,
}