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

use super::*;

#[test]
fn score_valid() {
    assert!(Score::MIN.is_valid());
    assert!(Score::MAX.is_valid());
    assert!(Score::MIN.is_min());
    assert!(!Score::MAX.is_min());
    assert!(!Score::MIN.is_max());
    assert!(Score::MAX.is_max());
    assert!(Score(*Score::MIN + *Score::MAX).is_valid());
    assert!(!Score(*Score::MIN - *Score::MAX).is_valid());
    assert!(Score(*Score::MIN - *Score::MAX).is_min());
    assert!(!Score(*Score::MAX + *Score::MAX).is_valid());
    assert!(Score(*Score::MAX + *Score::MAX).is_max());
}

#[test]
fn score_display() {
    assert_eq!("0.0%", format!("{}", Score::MIN));
    assert_eq!("100.0%", format!("{}", Score::MAX));
    assert_eq!("90.1%", format!("{}", Score(0.9012345)));
    assert_eq!("90.2%", format!("{}", Score(0.9015)));
}

#[test]
fn minmax_rating() {
    let owner1 = "a";
    let owner2 = "b";
    let owner3 = "c";
    let owner4 = "d";
    let ratings = vec![
        Rating {
            owner: Some(owner1.into()),
            score: 0.5.into(),
        },
        Rating {
            owner: None,
            score: 0.4.into(),
        },
        Rating {
            owner: Some(owner2.into()),
            score: 0.8.into(),
        },
        Rating {
            owner: Some(owner3.into()),
            score: 0.1.into(),
        },
    ];
    assert_eq!(None, Rating::minmax(&vec![], None));
    assert_eq!(None, Rating::minmax(&vec![], Some(owner1)));
    assert_eq!(None, Rating::minmax(&vec![], Some(owner4)));
    assert_eq!(
        Some((0.1.into(), 0.8.into())),
        Rating::minmax(&ratings, None)
    ); // all ratings
    assert_eq!(
        Some((0.4.into(), 0.5.into())),
        Rating::minmax(&ratings, Some(owner1))
    ); // anonymous and own rating
    assert_eq!(
        Some((0.4.into(), 0.8.into())),
        Rating::minmax(&ratings, Some(owner2))
    ); // anonymous and own rating
    assert_eq!(
        Some((0.1.into(), 0.4.into())),
        Rating::minmax(&ratings, Some(owner3))
    ); // anonymous and own rating
    assert_eq!(
        Some((0.4.into(), 0.4.into())),
        Rating::minmax(&ratings, Some(owner4))
    ); // only anonymous rating
}