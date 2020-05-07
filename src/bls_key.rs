use elrond_wasm::Vec;
use elrond_wasm::serde::ser::{Serialize, Serializer, SerializeTuple};
use elrond_wasm::serde::de::{Deserialize, Deserializer, Visitor, SeqAccess, Error};

// BLS keys have 128 bytes, signatures only 32
pub const BLS_KEY_BYTE_LENGTH: usize = 128;
pub const BLS_SIGNATURE_BYTE_LENGTH: usize = 32;

pub struct BLSKey(pub [u8; BLS_KEY_BYTE_LENGTH]);

impl BLSKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Serialize for BLSKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(BLS_KEY_BYTE_LENGTH)?;
        for i in 0..BLS_KEY_BYTE_LENGTH {
            seq.serialize_element(&self.0[i])?;
        }
        seq.end()
    }
}

struct BLSKeyVisitor;

impl<'a> Visitor<'a> for BLSKeyVisitor {
    type Value = BLSKey;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("BLSKey")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<BLSKey, A::Error>
        where A: SeqAccess<'a>
    {
        let mut arr = [0u8; BLS_KEY_BYTE_LENGTH];
        for i in 0..BLS_KEY_BYTE_LENGTH {
            arr[i] = seq.next_element()?
                .ok_or_else(|| Error::invalid_length(i, &self))?;
        }
        Ok(BLSKey(arr))
    }
}

impl<'de> Deserialize<'de> for BLSKey {
    fn deserialize<D>(deserializer: D) -> Result<BLSKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(BLS_KEY_BYTE_LENGTH, BLSKeyVisitor)
    }
}
