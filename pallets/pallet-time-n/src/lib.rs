#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    pub const BASE_CLOCK_HZ: u64 = 2_500_000_000;
    pub const DECIMAL_PLACES: u32 = 4;
    pub const FIELDS: usize = 6;

    #[derive(Clone, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum ClockStatus {
        Normal,
        Underclocked,
        Overclocked,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::storage]
    pub type NetTimeTicks<T> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    pub type BaseClockHz<T> =
        StorageValue<_, u64, ValueQuery, frame_support::traits::ConstU64<BASE_CLOCK_HZ>>;

    #[pallet::storage]
    pub type UnderclockTargetHz<T> =
        StorageValue<_, u64, ValueQuery, frame_support::traits::ConstU64<BASE_CLOCK_HZ>>;

    #[pallet::storage]
    pub type OverclockTargetHz<T> =
        StorageValue<_, u64, ValueQuery, frame_support::traits::ConstU64<BASE_CLOCK_HZ>>;

    #[pallet::storage]
    pub type TraderClockHz<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64>;

    #[pallet::storage]
    pub type TraderClockStatus<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ClockStatus>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        TimeAdvanced {
            ticks: u64,
            total: u128,
        },
        ClockTargetsSet {
            underclock_hz: u64,
            overclock_hz: u64,
        },
        TraderClockRecorded {
            trader: T::AccountId,
            measured_hz: u64,
            status: ClockStatus,
        },
        CheatDetected {
            trader: T::AccountId,
            status: ClockStatus,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        PermissionDenied,
        InvalidClockTarget,
        InvalidTick,
    }

    impl<T: Config> Pallet<T> {
        pub fn ticks_to_fields(ticks: u128) -> ([u16; 6], [u8; 4]) {
            let mut fields = [0u16; FIELDS];
            let mut decimals = [0u8; DECIMAL_PLACES as usize];
            let mut rem = ticks;
            for i in (0..DECIMAL_PLACES as usize).rev() {
                decimals[i] = (rem % 10) as u8;
                rem /= 10;
            }
            let mut intpart = rem;
            for i in (0..FIELDS).rev() {
                fields[i] = (intpart % (u16::MAX as u128 + 1)) as u16;
                intpart /= u16::MAX as u128 + 1;
            }
            (fields, decimals)
        }

        pub fn classify(measured: u64, under: u64, over: u64) -> ClockStatus {
            if measured < under {
                ClockStatus::Underclocked
            } else if measured > over {
                ClockStatus::Overclocked
            } else {
                ClockStatus::Normal
            }
        }

        pub fn net_time() -> u128 {
            NetTimeTicks::<T>::get()
        }

        fn ensure_permission(origin: OriginFor<T>) -> DispatchResult {
            if ensure_root(origin.clone()).is_ok() {
                return Ok(());
            }
            if ensure_signed(origin).is_ok() {
                return Ok(());
            }
            Err(Error::<T>::PermissionDenied.into())
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn advance_time(origin: OriginFor<T>, ticks: u64) -> DispatchResult {
            Self::ensure_permission(origin)?;
            if ticks == 0 {
                return Err(Error::<T>::InvalidTick.into());
            }
            NetTimeTicks::<T>::mutate(|t| *t = t.saturating_add(ticks as u128));
            let total = NetTimeTicks::<T>::get();
            Self::deposit_event(Event::TimeAdvanced { ticks, total });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn set_clock_targets(
            origin: OriginFor<T>,
            underclock_hz: u64,
            overclock_hz: u64,
        ) -> DispatchResult {
            Self::ensure_permission(origin)?;
            if underclock_hz > overclock_hz {
                return Err(Error::<T>::InvalidClockTarget.into());
            }
            UnderclockTargetHz::<T>::put(underclock_hz);
            OverclockTargetHz::<T>::put(overclock_hz);
            Self::deposit_event(Event::ClockTargetsSet {
                underclock_hz,
                overclock_hz,
            });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn record_trader_clock(
            origin: OriginFor<T>,
            trader: T::AccountId,
            measured_hz: u64,
        ) -> DispatchResult {
            Self::ensure_permission(origin)?;
            let under = UnderclockTargetHz::<T>::get();
            let over = OverclockTargetHz::<T>::get();
            let status = Self::classify(measured_hz, under, over);
            TraderClockHz::<T>::insert(trader.clone(), measured_hz);
            TraderClockStatus::<T>::insert(trader.clone(), status.clone());
            Self::deposit_event(Event::TraderClockRecorded {
                trader: trader.clone(),
                measured_hz,
                status: status.clone(),
            });
            if status != ClockStatus::Normal {
                Self::deposit_event(Event::CheatDetected {
                    trader,
                    status,
                });
            }
            Ok(())
        }

    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::pallet as pallet_time;
    use crate as pallet_time_n;

    use frame_support::{assert_ok, assert_noop, derive_impl, construct_runtime};
    use sp_io::TestExternalities;
    use sp_runtime::BuildStorage;

    type Block = frame_system::mocking::MockBlock<Test>;

    frame_support::construct_runtime!(
        pub enum Test {
            System: frame_system,
            Time: pallet,
        }
    );

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl frame_system::Config for Test {
        type Block = Block;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
    }

    fn new_test_ext() -> TestExternalities {
        frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
    }

    #[test]
    fn advance_accumulates() {
        new_test_ext().execute_with(|| {
            assert_ok!(pallet::Pallet::<Test>::advance_time(frame_system::RawOrigin::Root.into(), 5));
            assert_ok!(pallet::Pallet::<Test>::advance_time(frame_system::RawOrigin::Signed(1).into(), 7));
            assert_eq!(NetTimeTicks::<Test>::get(), 12);
            let ev = frame_system::Pallet::<Test>::events();
            let _ = ev;
        });
    }

    #[test]
    fn decimal_rollover() {
        new_test_ext().execute_with(|| {
            assert_ok!(pallet::Pallet::<Test>::advance_time(frame_system::RawOrigin::Root.into(), 10_000));
            let (fields, decimals) = pallet::Pallet::<Test>::ticks_to_fields(NetTimeTicks::<Test>::get());
            assert_eq!(decimals, [0, 0, 0, 0]);
            assert_eq!(fields[5], 1);
        });
    }

    #[test]
    fn clock_targets_inverted() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                pallet::Pallet::<Test>::set_clock_targets(frame_system::RawOrigin::Root.into(), 10, 5),
                Error::<Test>::InvalidClockTarget
            );
        });
    }

    #[test]
    fn trader_clock_flags() {
        new_test_ext().execute_with(|| {
            assert_ok!(pallet::Pallet::<Test>::set_clock_targets(frame_system::RawOrigin::Root.into(), 100, 200));
            assert_ok!(pallet::Pallet::<Test>::record_trader_clock(frame_system::RawOrigin::Root.into(), 1, 50));
            assert_eq!(TraderClockStatus::<Test>::get(1), Some(ClockStatus::Underclocked));
            assert_ok!(pallet::Pallet::<Test>::record_trader_clock(frame_system::RawOrigin::Root.into(), 2, 300));
            assert_eq!(TraderClockStatus::<Test>::get(2), Some(ClockStatus::Overclocked));
        });
    }

    #[test]
    fn invalid_tick() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                pallet::Pallet::<Test>::advance_time(frame_system::RawOrigin::Root.into(), 0),
                Error::<Test>::InvalidTick
            );
        });
    }
}
