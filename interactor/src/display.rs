//! Formatting helpers shared by the reporting commands.

use multiversx_sc_snippets::imports::*;

/// Formats a value expressed in hundredths of a percent (10000 = 100%) as e.g. "12.50%".
pub fn display_percentage<T>(value_per_10000: T) -> String
where
    T: Clone
        + core::ops::Div<u32, Output = T>
        + core::ops::Rem<u32, Output = T>
        + core::fmt::Display,
{
    let whole = value_per_10000.clone() / 100u32;
    let frac = value_per_10000 % 100u32;
    format!("{whole}.{frac:02}%")
}

/// Formats an atomic-unit amount as EGLD, right-aligned with 18 decimals.
pub fn display_egld_amount(managed_bu: &BigUint<StaticApi>) -> String {
    if managed_bu == &0u32 {
        return "       0".to_string();
    }

    display_egld_str(&managed_bu.to_alloc().to_string())
}

/// Same, for an amount that is already a plain big integer.
pub fn display_egld_rust_amount(amount: &RustBigUint) -> String {
    if amount == &RustBigUint::from(0u32) {
        return "       0".to_string();
    }

    display_egld_str(&amount.to_string())
}

fn display_egld_str(s: &str) -> String {
    let s = format!("{s:0>19}");
    let s = format!("{s:>26}");
    let len_before_dot = s.len() - 18;
    format!("{}.{}", &s[..len_before_dot], &s[len_before_dot..])
}
