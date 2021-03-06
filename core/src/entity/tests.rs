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

#[test]
fn default_uid() {
    assert!(!EntityUid::default().validate().is_ok());
    assert_eq!(
        EntityUid::default().as_ref().len(),
        mem::size_of::<EntityUid>()
    );
}

#[test]
fn generate_uid() {
    assert!(EntityUid::random().validate().is_ok());
}

#[test]
fn should_encode_decode_uid() {
    let uid = EntityUid::random();
    let encoded = uid.encode_to_string();
    let decoded = EntityUid::decode_from_str(&encoded).unwrap();
    assert_eq!(uid, decoded);
}

#[test]
fn should_fail_to_decode_too_long_string() {
    let uid = EntityUid::random();

    // Test encode -> decode roundtrip
    let mut encoded = uid.encode_to_string();
    assert!(EntityUid::decode_from_str(&encoded).is_ok());

    // Append one more character from the alphabet to the encoded string
    encoded.push(char::from('a'));
    assert!(EntityUid::decode_from_str(&encoded).is_err());
}

#[test]
fn should_fail_to_decode_too_short_string() {
    let uid = EntityUid::random();
    let mut encoded = uid.encode_to_string();
    encoded.truncate(EntityUid::MIN_STR_LEN - 1);
    assert!(EntityUid::decode_from_str(&encoded).is_err());
}

#[test]
fn rev_sequence() {
    let initial = EntityRevision::initial();
    assert!(initial.validate().is_ok());

    let next = initial.next();
    assert!(next.validate().is_ok());
    assert_ne!(EntityRevision::initial(), next);
    assert!(initial < next);

    let nextnext = next.next();
    assert!(nextnext.validate().is_ok());
    assert_ne!(EntityRevision::initial(), next);
    assert!(next < nextnext);
}

#[test]
fn hdr_without_uid() {
    let hdr = EntityHeader::initial_with_uid(EntityUid::default());
    assert!(!hdr.validate().is_ok());
    assert_eq!(EntityRevision::initial(), hdr.rev);
}

#[test]
fn should_generate_unique_initial_hdrs() {
    let hdr1 = EntityHeader::initial_random();
    let hdr2 = EntityHeader::initial_random();
    assert!(hdr1.validate().is_ok());
    assert_eq!(EntityRevision::initial(), hdr1.rev);
    assert!(hdr2.validate().is_ok());
    assert_eq!(EntityRevision::initial(), hdr2.rev);
    assert_ne!(hdr1.uid, hdr2.uid);
    assert_eq!(hdr1.rev, hdr2.rev);
}
