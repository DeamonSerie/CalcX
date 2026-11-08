#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::{FixedU128, FixedPointNumber, Saturating};
    use sp_std::vec::Vec;

    pub type Price = FixedU128;

    #[derive(Clone, Debug, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct TradeSchedule {
        pub window_blocks: u32,
        pub base_tick_bps: u32,
        pub volatility_k: u32,
        pub min_window: Price,
    }

    #[derive(Clone, Debug, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct ChangeWindow<T: Config> {
        pub id: u64,
        pub asset: T::AssetId,
        pub opened_at: BlockNumberFor<T>,
        pub closes_at: BlockNumberFor<T>,
        pub price_lo_y: Price,
        pub price_hi_y: Price,
        pub price_lo_x: Price,
        pub price_hi_x: Price,
        pub trade_magnitude: Price,
        pub verified: bool,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type AssetId: Parameter + Member + Copy + MaybeSerializeDeserialize + MaxEncodedLen;
        type MinWindow: Get<Price>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type TradeSchedules<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, TradeSchedule>;

    #[pallet::storage]
    pub type Maintainers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>;

    #[pallet::storage]
    pub type NextWindowId<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    pub type Windows<T: Config> = StorageMap<_, Blake2_128Concat, u64, ChangeWindow<T>>;

    #[pallet::storage]
    pub type WindowOrder<T> = StorageValue<_, BoundedVec<u64, ConstU32<100000>>, ValueQuery>;

    #[pallet::storage]
    pub type ActiveWindow<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, u64>;

    #[pallet::storage]
    pub type RegisteredAssets<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, ()>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ScheduleSet { asset: T::AssetId },
        WindowOpened { id: u64, asset: T::AssetId },
        Traded { asset: T::AssetId, amount: u128, window: u64 },
        WindowClosed { id: u64 },
        AssetRegistered { asset: T::AssetId },
        MaintainerAdded { who: T::AccountId },
        MaintainerRemoved { who: T::AccountId },
        WindowVerified { id: u64 },
    }

    #[pallet::error]
    pub enum Error<T> {
        NotMaintainer,
        NonDecimalNotAllowed,
        NoSchedule,
        NoActiveWindow,
        OutsideWindow,
        WindowNotFound,
        AlreadyRegistered,
        InvalidBounds,
    }

    impl<T: Config> Pallet<T> {
        fn is_maintainer(who: &T::AccountId) -> bool {
            Maintainers::<T>::contains_key(who)
        }

        fn ensure_auth(origin: OriginFor<T>) -> DispatchResult {
            if ensure_root(origin.clone()).is_ok() || ensure_none(origin.clone()).is_ok() {
                return Ok(());
            }
            let who = ensure_signed(origin)?;
            if Self::is_maintainer(&who) {
                Ok(())
            } else {
                Err(Error::<T>::NotMaintainer.into())
            }
        }

        fn has_fraction(p: Price) -> bool {
            p.frac() != Price::from_inner(0)
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn set_trade_schedule(
            origin: OriginFor<T>,
            asset: T::AssetId,
            schedule: TradeSchedule,
        ) -> DispatchResult {
            // root or maintainer
            Self::ensure_auth(origin.clone())?;
            let _ = origin;
            if !Self::has_fraction(schedule.min_window) {
                return Err(Error::<T>::NonDecimalNotAllowed.into());
            }
            if schedule.window_blocks == 0 {
                return Err(Error::<T>::InvalidBounds.into());
            }
            TradeSchedules::<T>::insert(asset, schedule);
            Self::deposit_event(Event::ScheduleSet { asset });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn open_window(
            origin: OriginFor<T>,
            asset: T::AssetId,
        ) -> DispatchResult {
            Self::ensure_auth(origin.clone())?;
            let schedule = TradeSchedules::<T>::get(asset).ok_or(Error::<T>::NoSchedule)?;
            if !Self::has_fraction(schedule.min_window) {
                return Err(Error::<T>::NonDecimalNotAllowed.into());
            }
            let lo = Price::from_rational(5001u128, 1000u128);
            let hi = Price::from_rational(5002u128, 1000u128);
            if !Self::has_fraction(lo) || !Self::has_fraction(hi) {
                return Err(Error::<T>::NonDecimalNotAllowed.into());
            }
            let now = <frame_system::Pallet<T>>::block_number();
            let closes = now + schedule.window_blocks.into();
            let id = NextWindowId::<T>::get();
            NextWindowId::<T>::put(id + 1);
            let window = ChangeWindow::<T> {
                id,
                asset,
                opened_at: now,
                closes_at: closes,
                price_lo_y: lo,
                price_hi_y: hi,
                price_lo_x: lo,
                price_hi_x: hi,
                trade_magnitude: Price::from_rational(1u128, 1000u128),
                verified: false,
            };
            Windows::<T>::insert(id, window);
            WindowOrder::<T>::mutate(|v| { let _ = v.try_push(id); });
            ActiveWindow::<T>::insert(asset, id);
            Self::deposit_event(Event::WindowOpened { id, asset });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn trade(
            origin: OriginFor<T>,
            asset: T::AssetId,
            amount: u128,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            let id = ActiveWindow::<T>::get(asset).ok_or(Error::<T>::NoActiveWindow)?;
            let mut window = Windows::<T>::get(id).ok_or(Error::<T>::WindowNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();
            if now < window.opened_at || now > window.closes_at {
                return Err(Error::<T>::OutsideWindow.into());
            }
            // enforce decimal on bounds (already)
            if !Self::has_fraction(window.price_lo_y) || !Self::has_fraction(window.price_hi_y)
                || !Self::has_fraction(window.price_lo_x) || !Self::has_fraction(window.price_hi_x) {
                return Err(Error::<T>::NonDecimalNotAllowed.into());
            }
            window.trade_magnitude = Saturating::saturating_add(window.trade_magnitude, Price::from_rational(1u128, 1000u128));
            let _ = amount;
            Windows::<T>::insert(id, window.clone());
            Self::deposit_event(Event::Traded { asset, amount, window: id });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn close_window(
            origin: OriginFor<T>,
            asset: T::AssetId,
        ) -> DispatchResult {
            Self::ensure_auth(origin.clone())?;
            let _ = origin;
            let id = ActiveWindow::<T>::take(asset).ok_or(Error::<T>::NoActiveWindow)?;
            let mut window = Windows::<T>::get(id).ok_or(Error::<T>::WindowNotFound)?;
            window.verified = false;
            Windows::<T>::insert(id, window);
            // compute next size (advisory) - placeholder
            let _ = TradeSchedules::<T>::get(asset);
            Self::deposit_event(Event::WindowClosed { id });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn register_asset(
            origin: OriginFor<T>,
            asset: T::AssetId,
        ) -> DispatchResult {
            // allow root/maintainer; signed users? We'll allow root/maintainer only.
            Self::ensure_auth(origin.clone())?;
            let _ = origin;
            if RegisteredAssets::<T>::contains_key(asset) {
                return Err(Error::<T>::AlreadyRegistered.into());
            }
            RegisteredAssets::<T>::insert(asset, ());
            if !TradeSchedules::<T>::contains_key(asset) {
                TradeSchedules::<T>::insert(asset, TradeSchedule {
                    window_blocks: 10,
                    base_tick_bps: 10,
                    volatility_k: 1,
                    min_window: T::MinWindow::get(),
                });
            }
            Self::deposit_event(Event::AssetRegistered { asset });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn add_maintainer(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            Maintainers::<T>::insert(who.clone(), ());
            Self::deposit_event(Event::MaintainerAdded { who });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn remove_maintainer(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            Maintainers::<T>::remove(who.clone());
            Self::deposit_event(Event::MaintainerRemoved { who });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn verify_window(origin: OriginFor<T>, id: u64) -> DispatchResult {
            ensure_root(origin)?;
            Windows::<T>::try_mutate(id, |w| {
                let w = w.as_mut().ok_or(Error::<T>::WindowNotFound)?;
                w.verified = true;
                Ok::<(), DispatchError>(())
            })?;
            Self::deposit_event(Event::WindowVerified { id });
            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
