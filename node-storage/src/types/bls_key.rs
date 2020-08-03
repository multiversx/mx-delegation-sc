use elrond_wasm::Box;
use elrond_wasm::elrond_codec::*;
use elrond_wasm::{H256, SCResult, MultiArg2, Vec};

// BLS keys have 96 bytes, signatures only 32
pub const BLS_KEY_BYTE_LENGTH: usize = 96;
pub const BLS_SIGNATURE_BYTE_LENGTH: usize = 32;

pub struct BLSKey(pub Box<[u8; BLS_KEY_BYTE_LENGTH]>);
pub type BLSSignature = H256;

pub type BLSStatusMultiArg = MultiArg2<BLSKey, i32>;

impl BLSKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn from_array(arr: [u8; BLS_KEY_BYTE_LENGTH]) -> Self {
        BLSKey(Box::new(arr))
    }

    pub fn from_bytes(bytes: &[u8]) -> SCResult<Self> {
        if bytes.len() != BLS_KEY_BYTE_LENGTH {
            return sc_error!("bad BLS key length");
        }
        let mut arr = [0u8; BLS_KEY_BYTE_LENGTH];
        for (i, &b) in bytes.iter().enumerate() {
            arr[i] = b;
        }
        SCResult::Ok(BLSKey(Box::new(arr)))
    }
}

impl Encode for BLSKey {
    #[inline]
    fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        dest.write(&self.0[..]);
        Ok(())
    }
}

impl Decode for BLSKey {
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        let mut boxed = Box::new([0u8; BLS_KEY_BYTE_LENGTH]);
        input.read_into(boxed.as_mut())?;
        Ok(BLSKey(boxed))
    }
}
