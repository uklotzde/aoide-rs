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

use failure::Error;

use mime;

use rmp_serde;

use serde_cbor;

use serde_json;

///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SerializationFormat {
    JSON = 1,
    CBOR = 2,
    MessagePack = 3,
}

impl SerializationFormat {
    pub fn from(from: i16) -> Option<Self> {
        if from == (SerializationFormat::JSON as i16) {
            Some(SerializationFormat::JSON)
        } else if from == (SerializationFormat::CBOR as i16) {
            Some(SerializationFormat::CBOR)
        } else if from == (SerializationFormat::MessagePack as i16) {
            Some(SerializationFormat::MessagePack)
        } else {
            None
        }
    }

    pub fn from_media_type(media_type: &mime::Mime) -> Option<Self> {
        match (media_type.type_(), media_type.subtype()) {
            (mime::APPLICATION, mime::JSON) => Some(SerializationFormat::JSON),
            (mime::APPLICATION, mime::MSGPACK) => Some(SerializationFormat::MessagePack),
            (mime::APPLICATION, subtype) => {
                if subtype.as_str() == "cbor" {
                    Some(SerializationFormat::CBOR)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Into<mime::Mime> for SerializationFormat {
    fn into(self) -> mime::Mime {
        match self {
            SerializationFormat::JSON => mime::APPLICATION_JSON,
            SerializationFormat::CBOR => "application/cbor".parse::<mime::Mime>().unwrap(),
            SerializationFormat::MessagePack => mime::APPLICATION_MSGPACK,
            //_ => mime::APPLICATION_OCTET_STREAM,
        }
    }
}

pub(crate) fn serialize_with_format<T>(
    entity: &T,
    format: SerializationFormat,
) -> Result<Vec<u8>, Error>
where
    T: serde::Serialize,
{
    let blob = match format {
        SerializationFormat::JSON => serde_json::to_vec(entity)?,
        SerializationFormat::CBOR => serde_cbor::to_vec(entity)?,
        SerializationFormat::MessagePack => rmp_serde::to_vec(entity)?,
        //_ => return Err(failure::format_err!("Unsupported format for serialization: {:?}", format))
    };
    Ok(blob)
}

pub(crate) fn deserialize_slice_with_format<'a, T>(
    slice: &'a [u8],
    format: SerializationFormat,
) -> Result<T, Error>
where
    T: serde::Deserialize<'a>,
{
    let deserialized = match format {
        SerializationFormat::JSON => serde_json::from_slice::<T>(slice)?,
        SerializationFormat::CBOR => serde_cbor::from_slice::<T>(slice)?,
        SerializationFormat::MessagePack => rmp_serde::from_slice::<T>(slice)?,
        //_ => return Err(failure::format_err!("Unsupported format for deserialization: {:?}", format))
    };
    Ok(deserialized)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerializedEntity {
    pub header: EntityHeader,

    pub format: SerializationFormat,

    pub version: EntityVersion,

    pub blob: Vec<u8>,
}

impl SerializedEntity {
    pub fn slice_to_json_array(serialized_entities: &[SerializedEntity]) -> Result<Vec<u8>, Error> {
        let mut json_array = Vec::with_capacity(
            serialized_entities
                .iter()
                .fold(serialized_entities.len() + 1, |acc, ref item| {
                    acc + item.blob.len()
                }),
        );
        json_array.extend_from_slice(b"[");
        for (i, item) in serialized_entities.iter().enumerate() {
            if item.format != SerializationFormat::JSON {
                let e = failure::format_err!("Unsupported serialization format while loading multiple entities: expected = {:?}, actual = {:?}", SerializationFormat::JSON, item.format);
                return Err(e);
            }
            if i > 0 {
                json_array.extend_from_slice(b",");
            }
            json_array.extend_from_slice(&item.blob);
        }
        json_array.extend_from_slice(b"]");
        Ok(json_array)
    }

    pub fn deserialize<'a, T>(&'a self) -> Result<T, Error>
    where
        T: serde::Deserialize<'a>,
    {
        deserialize_slice_with_format(&self.blob, self.format)
    }
}
