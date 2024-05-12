use multiversx_sc::codec::*;
use multiversx_sc::{Box, Vec};

multiversx_sc::derive_imports!();

/// BLS signatures have 48 bytes
pub const BLS_SIGNATURE_BYTE_LENGTH: usize = 48;

#[type_abi]
pub struct BLSSignature(pub Box<[u8; BLS_SIGNATURE_BYTE_LENGTH]>);

impl BLSSignature {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn from_array(arr: [u8; BLS_SIGNATURE_BYTE_LENGTH]) -> Self {
        BLSSignature(Box::new(arr))
    }
}

impl NestedEncode for BLSSignature {
    #[inline]
    fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.0.dep_encode(dest)
    }

    #[inline]
    fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
        &self,
        dest: &mut O,
        c: ExitCtx,
        exit: fn(ExitCtx, EncodeError) -> !,
    ) {
        self.0.dep_encode_or_exit(dest, c, exit);
    }
}

impl TopEncode for BLSSignature {
    #[inline]
    fn top_encode<O: TopEncodeOutput>(&self, output: O) -> Result<(), EncodeError> {
        self.0.top_encode(output)
    }

    #[inline]
    fn top_encode_or_exit<O: TopEncodeOutput, ExitCtx: Clone>(
        &self,
        output: O,
        c: ExitCtx,
        exit: fn(ExitCtx, EncodeError) -> !,
    ) {
        self.0.top_encode_or_exit(output, c, exit);
    }
}

impl NestedDecode for BLSSignature {
    #[inline]
    fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(BLSSignature(
            Box::<[u8; BLS_SIGNATURE_BYTE_LENGTH]>::dep_decode(input)?,
        ))
    }

    fn dep_decode_or_exit<I: NestedDecodeInput, ExitCtx: Clone>(
        input: &mut I,
        c: ExitCtx,
        exit: fn(ExitCtx, DecodeError) -> !,
    ) -> Self {
        BLSSignature(Box::<[u8; BLS_SIGNATURE_BYTE_LENGTH]>::dep_decode_or_exit(
            input, c, exit,
        ))
    }
}

impl TopDecode for BLSSignature {
    #[inline]
    fn top_decode<I: TopDecodeInput>(input: I) -> Result<Self, DecodeError> {
        Ok(BLSSignature(
            Box::<[u8; BLS_SIGNATURE_BYTE_LENGTH]>::top_decode(input)?,
        ))
    }

    #[inline]
    fn top_decode_or_exit<I: TopDecodeInput, ExitCtx: Clone>(
        input: I,
        c: ExitCtx,
        exit: fn(ExitCtx, DecodeError) -> !,
    ) -> Self {
        BLSSignature(Box::<[u8; BLS_SIGNATURE_BYTE_LENGTH]>::top_decode_or_exit(
            input, c, exit,
        ))
    }
}

impl PartialEq for BLSSignature {
    #[allow(clippy::op_ref)]
    fn eq(&self, other: &Self) -> bool {
        &self.0[..] == &other.0[..]
    }
}

// only needed for tests
use core::fmt;
impl fmt::Debug for BLSSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use multiversx_sc::codec::test_util::*;
    use multiversx_sc::Vec;

    #[test]
    fn test_bls_serialization() {
        let bls_sig = BLSSignature::from_array([4u8; BLS_SIGNATURE_BYTE_LENGTH]);
        let expected_bytes: &[u8] = &[4u8; BLS_SIGNATURE_BYTE_LENGTH];

        // serialize
        let serialized_bytes = check_top_encode(&bls_sig);
        assert_eq!(serialized_bytes.as_slice(), expected_bytes);

        // deserialize
        let deserialized: BLSSignature = check_top_decode::<BLSSignature>(&serialized_bytes[..]);
        assert_eq!(deserialized.to_vec(), bls_sig.to_vec());
    }

    #[test]
    fn test_vec_bls_serialization() {
        let mut bls_vec: Vec<BLSSignature> = Vec::new();
        for _ in 0..3 {
            bls_vec.push(BLSSignature::from_array([4u8; BLS_SIGNATURE_BYTE_LENGTH]));
        }
        let expected_bytes: &[u8] = &[4u8; BLS_SIGNATURE_BYTE_LENGTH * 3];

        // serialize
        let serialized_bytes = check_top_encode(&bls_vec);
        assert_eq!(serialized_bytes.as_slice(), expected_bytes);

        // deserialize
        let deserialized: Vec<BLSSignature> =
            check_top_decode::<Vec<BLSSignature>>(&serialized_bytes[..]);
        assert_eq!(deserialized.len(), bls_vec.len());
        for i in 0..3 {
            assert_eq!(deserialized[i].to_vec(), bls_vec[i].to_vec());
        }
    }
}
