use elrond_wasm::*;

/// Takes 2 separate vecs and combines them into a single vec, alternating elements from the first with elements from the second.
/// Assumes vectors have the same length.
/// E.g. zip_vectors([1, 2, 3], [4, 5, 6]) -> [1, 4, 2, 5, 3, 6]
pub fn zip_vectors(
        mut first_vec: Vec<Vec<u8>>,
        mut second_vec: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    
    let len = first_vec.len();
    let mut zipped = Vec::with_capacity(len * 2);
    zipped.resize_with(len * 2, Default::default);
    let mut i: isize = (len as isize) - 1;
    // we use remove to move ownership of the elements and avoid a clone
    // we go backwards to keep Vec::remove O(1)
    while i >= 0 {
        let i_usize = i as usize;
        zipped[i_usize*2] = first_vec.remove(i_usize);
        zipped[i_usize*2+1] = second_vec.remove(i_usize);
        i -= 1;
    }
    zipped
}