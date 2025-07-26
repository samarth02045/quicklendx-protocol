pub fn calculate_profit(
    investment_amount: i128,
    payment_amount: i128,
    platform_fee_bps: i128,
) -> (i128, i128) {
    let profit = payment_amount - investment_amount;
    let platform_fee = profit * platform_fee_bps / 10_000;
    let investor_return = payment_amount - platform_fee;
    (investor_return, platform_fee)
}
