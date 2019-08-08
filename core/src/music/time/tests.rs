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

#[test]
fn default_time_sig() {
    assert!(TimeSignature::default().validate().is_err());
}

#[test]
fn new_time_sig() {
    assert!(TimeSignature::new(0, 0).validate().is_err());
    assert!(TimeSignature::new(0, 1).validate().is_err());
    assert!(TimeSignature::new(1, 0).validate().is_err());
    assert!(TimeSignature::new(1, 1).validate().is_ok());
    assert!(TimeSignature::new(3, 4).validate().is_ok());
    assert!(TimeSignature::new(4, 4).validate().is_ok());
    assert!(TimeSignature::new(4, 3).validate().is_ok());
}
