use multiversx_sc::arrayvec::ArrayVec;

use super::partition_dedup::partition_dedup;

pub const MAX_AFFECTED_USERS: usize = 10000;
pub type AffectedUserIdVec = ArrayVec<usize, MAX_AFFECTED_USERS>;

pub fn affected_users_sort_dedup(affected_users: &mut AffectedUserIdVec) {
    affected_users.sort_unstable();
    let (dedup, _) = partition_dedup(affected_users);
    let dedup_len = dedup.len();
    unsafe {
        affected_users.set_len(dedup_len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::TryFrom;

    #[test]
    fn test_affected_users_sort_dedup_empty() {
        let mut affected_users = AffectedUserIdVec::new();
        affected_users_sort_dedup(&mut affected_users);
        assert!(affected_users.is_empty());
    }

    #[test]
    fn test_affected_users_sort_dedup_singleton() {
        let mut affected_users = AffectedUserIdVec::try_from(&[1][..]).unwrap();
        affected_users_sort_dedup(&mut affected_users);
        assert_eq!(affected_users.as_slice(), [1]);
    }

    #[test]
    fn test_affected_users_sort_dedup_more() {
        let mut affected_users =
            AffectedUserIdVec::try_from(&[1, 2, 1, 3, 3, 1, 2, 1][..]).unwrap();
        affected_users_sort_dedup(&mut affected_users);
        assert_eq!(affected_users.as_slice(), [1, 2, 3]);
    }
}
