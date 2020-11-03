use elrond_wasm::Box;
use elrond_wasm::elrond_codec::*;
use elrond_wasm::{SCResult, Vec};

// BLS signatures have 48 bytes
pub const BLS_SIGNATURE_BYTE_LENGTH: usize = 48;

pub struct BLSSignature(pub Box<[u8; BLS_SIGNATURE_BYTE_LENGTH]>);

impl BLSSignature {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn from_array(arr: [u8; BLS_SIGNATURE_BYTE_LENGTH]) -> Self {
        BLSSignature(Box::new(arr))
    }

    pub fn from_bytes(bytes: &[u8]) -> SCResult<Self> {
        require!(bytes.len() == BLS_SIGNATURE_BYTE_LENGTH, "bad BLS signature length");
        
        let mut arr = [0u8; BLS_SIGNATURE_BYTE_LENGTH];
        for (i, &b) in bytes.iter().enumerate() {
            arr[i] = b;
        }
        SCResult::Ok(BLSSignature(Box::new(arr)))
    }
}

impl Encode for BLSSignature {
    #[inline]
    fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        dest.write(&self.0[..]);
        Ok(())
    }
}

impl Decode for BLSSignature {
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        let mut boxed = Box::new([0u8; BLS_SIGNATURE_BYTE_LENGTH]);
        input.read_into(boxed.as_mut())?;
        Ok(BLSSignature(boxed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elrond_wasm::Vec;

    #[test]
    fn test_bls_serialization() {
        let bls_sig = BLSSignature::from_array([4u8; BLS_SIGNATURE_BYTE_LENGTH]);
        let expected_bytes: &[u8] = &[4u8; BLS_SIGNATURE_BYTE_LENGTH];

        // serialize
        let serialized_bytes = bls_sig.top_encode().unwrap();
        assert_eq!(serialized_bytes.as_slice(), expected_bytes);

        // deserialize
        let deserialized: BLSSignature = decode_from_byte_slice(&serialized_bytes[..]).unwrap();
        assert_eq!(deserialized.to_vec(), bls_sig.to_vec());
    }

    #[test]
    fn test_vec_bls_serialization() {
        let mut bls_vec: Vec<BLSSignature> = Vec::new();
        for _ in 0..3 {
            bls_vec.push(BLSSignature::from_array([4u8; BLS_SIGNATURE_BYTE_LENGTH]));
        }
        let expected_bytes: &[u8] = &[4u8; BLS_SIGNATURE_BYTE_LENGTH*3];

        // serialize
        let serialized_bytes = bls_vec.top_encode().unwrap();
        assert_eq!(serialized_bytes.as_slice(), expected_bytes);

        // deserialize
        let deserialized: Vec<BLSSignature> = decode_from_byte_slice(serialized_bytes.as_slice()).unwrap();
        assert_eq!(deserialized.len(), bls_vec.len());
        for i in 0..3 {
            assert_eq!(deserialized[i].to_vec(), bls_vec[i].to_vec());
        }
    }
}
