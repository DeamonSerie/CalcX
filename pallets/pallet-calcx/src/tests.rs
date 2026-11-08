#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::*;
    use frame_support::assert_ok;
    use sp_runtime::FixedU128;

    #[test]
    fn set_schedule_and_reject_non_decimal() {
        new_test_ext().execute_with(|| {
            let sched = TradeSchedule {
                window_blocks: 10,
                base_tick_bps: 10,
                volatility_k: 1,
                min_window: FixedU128::from_rational(1001u128, 1000u128),
            };
            assert_ok!(Pallet::<Test>::set_trade_schedule(
                RuntimeOrigin::root(),
                1u32,
                sched
            ));
            let bad = TradeSchedule {
                window_blocks: 10,
                base_tick_bps: 10,
                volatility_k: 1,
                min_window: FixedU128::from(1u128),
            };
            assert!(Pallet::<Test>::set_trade_schedule(
                RuntimeOrigin::root(),
                2u32,
                bad
            ).is_err());
        });
    }

    #[test]
    fn register_asset_assigns_schedule() {
        new_test_ext().execute_with(|| {
            assert_ok!(Pallet::<Test>::register_asset(RuntimeOrigin::root(), 5u32));
            assert!(TradeSchedules::<Test>::contains_key(5u32));
        });
    }

    #[test]
    fn open_trade_close_window() {
        new_test_ext().execute_with(|| {
            assert_ok!(Pallet::<Test>::register_asset(RuntimeOrigin::root(), 1u32));
            assert_ok!(Pallet::<Test>::open_window(RuntimeOrigin::root(), 1u32));
            let id = ActiveWindow::<Test>::get(1u32).unwrap();
            assert_ok!(Pallet::<Test>::trade(
                RuntimeOrigin::signed(1),
                1u32,
                10u128
            ));
            assert_ok!(Pallet::<Test>::close_window(RuntimeOrigin::root(), 1u32));
            assert!(ActiveWindow::<Test>::get(1u32).is_none());
            let w = Windows::<Test>::get(id).unwrap();
            assert!(!w.verified);
        });
    }

    #[test]
    fn trade_outside_window_rejected() {
        new_test_ext().execute_with(|| {
            assert_ok!(Pallet::<Test>::register_asset(RuntimeOrigin::root(), 1u32));
            // no active window
            assert!(Pallet::<Test>::trade(RuntimeOrigin::signed(1), 1u32, 10u128).is_err());
        });
    }
}
