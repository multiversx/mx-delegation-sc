use user_fund_storage::types::*;

elrond_wasm::imports!();

pub fn check_consistency_for_type<M>(module: &M, fund_type: FundType)
where
    M: user_fund_storage::fund_module::FundModule,
    M::BigInt: elrond_wasm::api::BigIntApi,
    for<'a, 'b> &'a M::BigUint: core::ops::Add<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::Sub<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::Mul<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::Div<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::Rem<&'b M::BigUint, Output = M::BigUint>,
    for<'b> M::BigUint: core::ops::AddAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::SubAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::MulAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::DivAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::RemAssign<&'b M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::BitAnd<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::BitOr<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::BitXor<&'b M::BigUint, Output = M::BigUint>,
    for<'b> M::BigUint: core::ops::BitAndAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::BitOrAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::BitXorAssign<&'b M::BigUint>,
    for<'a> &'a M::BigUint: core::ops::Shr<usize, Output = M::BigUint>,
    for<'a> &'a M::BigUint: core::ops::Shl<usize, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigInt: core::ops::Add<&'b M::BigInt, Output = M::BigInt>,
    for<'a, 'b> &'a M::BigInt: core::ops::Sub<&'b M::BigInt, Output = M::BigInt>,
    for<'a, 'b> &'a M::BigInt: core::ops::Mul<&'b M::BigInt, Output = M::BigInt>,
    for<'a, 'b> &'a M::BigInt: core::ops::Div<&'b M::BigInt, Output = M::BigInt>,
    for<'a, 'b> &'a M::BigInt: core::ops::Rem<&'b M::BigInt, Output = M::BigInt>,
    for<'b> M::BigInt: core::ops::AddAssign<&'b M::BigInt>,
    for<'b> M::BigInt: core::ops::SubAssign<&'b M::BigInt>,
    for<'b> M::BigInt: core::ops::MulAssign<&'b M::BigInt>,
    for<'b> M::BigInt: core::ops::DivAssign<&'b M::BigInt>,
    for<'b> M::BigInt: core::ops::RemAssign<&'b M::BigInt>,
{
    let mut sum = M::BigUint::zero();
    let type_list = module.get_fund_list_by_type(fund_type);
    let mut id = type_list.first;
    let mut prev_id = 0;
    while id > 0 {
        let fund_item = module.fund_by_id(id).get();

        // check next/prev
        assert_eq!(
            fund_item.type_list_prev, prev_id,
            "type list inconsistency: bad prev"
        );
        if fund_item.type_list_next == 0 {
            // println!("last: {}  id: {}", type_list.last, id);
            assert_eq!(type_list.last, id, "type list inconsistency: bad last");
        }

        sum += &fund_item.balance;
        prev_id = id;
        id = fund_item.type_list_next;
    }

    assert!(
        sum == type_list.total_balance,
        "type list inconsistency: bad sum"
    );
}

pub fn check_consistency_for_user_type<M>(module: &M, user_id: usize, fund_type: FundType)
where
    M: user_fund_storage::fund_module::FundModule,
    M::BigInt: elrond_wasm::api::BigIntApi,
    for<'a, 'b> &'a M::BigUint: core::ops::Add<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::Sub<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::Mul<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::Div<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::Rem<&'b M::BigUint, Output = M::BigUint>,
    for<'b> M::BigUint: core::ops::AddAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::SubAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::MulAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::DivAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::RemAssign<&'b M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::BitAnd<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::BitOr<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::BitXor<&'b M::BigUint, Output = M::BigUint>,
    for<'b> M::BigUint: core::ops::BitAndAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::BitOrAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::BitXorAssign<&'b M::BigUint>,
    for<'a> &'a M::BigUint: core::ops::Shr<usize, Output = M::BigUint>,
    for<'a> &'a M::BigUint: core::ops::Shl<usize, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigInt: core::ops::Add<&'b M::BigInt, Output = M::BigInt>,
    for<'a, 'b> &'a M::BigInt: core::ops::Sub<&'b M::BigInt, Output = M::BigInt>,
    for<'a, 'b> &'a M::BigInt: core::ops::Mul<&'b M::BigInt, Output = M::BigInt>,
    for<'a, 'b> &'a M::BigInt: core::ops::Div<&'b M::BigInt, Output = M::BigInt>,
    for<'a, 'b> &'a M::BigInt: core::ops::Rem<&'b M::BigInt, Output = M::BigInt>,
    for<'b> M::BigInt: core::ops::AddAssign<&'b M::BigInt>,
    for<'b> M::BigInt: core::ops::SubAssign<&'b M::BigInt>,
    for<'b> M::BigInt: core::ops::MulAssign<&'b M::BigInt>,
    for<'b> M::BigInt: core::ops::DivAssign<&'b M::BigInt>,
    for<'b> M::BigInt: core::ops::RemAssign<&'b M::BigInt>,
{
    let mut sum = M::BigUint::zero();
    let user_type_list = module.fund_list_by_user(user_id, fund_type).get();
    let mut id = user_type_list.first;
    let mut prev_id = 0;
    while id > 0 {
        let fund_item = module.fund_by_id(id).get();

        assert_eq!(
            fund_item.user_id, user_id,
            "user-type list inconsistency: bad user_id"
        );

        // check next/prev
        assert_eq!(
            fund_item.user_list_prev, prev_id,
            "user-type list inconsistency: bad prev"
        );
        if fund_item.user_list_next == 0 {
            assert_eq!(
                user_type_list.last, id,
                "user-type list inconsistency: bad list last"
            );
        }

        sum += &fund_item.balance;
        prev_id = id;
        id = fund_item.user_list_next;
    }

    assert!(
        sum == user_type_list.total_balance,
        "user-type list inconsistency: bad sum"
    );
}

pub fn check_consistency<M>(module: &M, num_users: usize)
where
    M: user_fund_storage::fund_module::FundModule,
    M::BigInt: elrond_wasm::api::BigIntApi,
    for<'a, 'b> &'a M::BigUint: core::ops::Add<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::Sub<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::Mul<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::Div<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::Rem<&'b M::BigUint, Output = M::BigUint>,
    for<'b> M::BigUint: core::ops::AddAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::SubAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::MulAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::DivAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::RemAssign<&'b M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::BitAnd<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::BitOr<&'b M::BigUint, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigUint: core::ops::BitXor<&'b M::BigUint, Output = M::BigUint>,
    for<'b> M::BigUint: core::ops::BitAndAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::BitOrAssign<&'b M::BigUint>,
    for<'b> M::BigUint: core::ops::BitXorAssign<&'b M::BigUint>,
    for<'a> &'a M::BigUint: core::ops::Shr<usize, Output = M::BigUint>,
    for<'a> &'a M::BigUint: core::ops::Shl<usize, Output = M::BigUint>,
    for<'a, 'b> &'a M::BigInt: core::ops::Add<&'b M::BigInt, Output = M::BigInt>,
    for<'a, 'b> &'a M::BigInt: core::ops::Sub<&'b M::BigInt, Output = M::BigInt>,
    for<'a, 'b> &'a M::BigInt: core::ops::Mul<&'b M::BigInt, Output = M::BigInt>,
    for<'a, 'b> &'a M::BigInt: core::ops::Div<&'b M::BigInt, Output = M::BigInt>,
    for<'a, 'b> &'a M::BigInt: core::ops::Rem<&'b M::BigInt, Output = M::BigInt>,
    for<'b> M::BigInt: core::ops::AddAssign<&'b M::BigInt>,
    for<'b> M::BigInt: core::ops::SubAssign<&'b M::BigInt>,
    for<'b> M::BigInt: core::ops::MulAssign<&'b M::BigInt>,
    for<'b> M::BigInt: core::ops::DivAssign<&'b M::BigInt>,
    for<'b> M::BigInt: core::ops::RemAssign<&'b M::BigInt>,
{
    for &fund_type in FundType::ALL_TYPES.iter() {
        check_consistency_for_type(module, fund_type);

        for user_id in 1..(num_users + 1) {
            check_consistency_for_user_type(module, user_id, fund_type);
        }
    }
}
