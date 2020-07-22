use elrond_wasm::Vec;
use elrond_wasm::esd_light::*;

// BLS keys have 96 bytes, signatures only 32
pub const BLS_KEY_BYTE_LENGTH: usize = 96;
pub const BLS_SIGNATURE_BYTE_LENGTH: usize = 32;

pub struct BLSKey(pub [u8; BLS_KEY_BYTE_LENGTH]);

impl BLSKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Encode for BLSKey {
    #[inline]
    fn dep_encode_to<O: Output>(&self, dest: &mut O) {
        dest.write(&self.0[..]);
    }
}

impl Decode for BLSKey {
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        let mut arr = [0u8; BLS_KEY_BYTE_LENGTH];
        input.read_into(&mut arr)?;
        Ok(BLSKey(arr))
    }
}
