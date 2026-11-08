#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use pallet_balances;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_balances::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        #[pallet::constant]
        type CxPerTx: Get<u128>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Rewarded { author: T::AccountId, amount: u128 },
    }

    #[pallet::error]
    pub enum Error<T> {
        None,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(_n: BlockNumberFor<T>) {
            // minting hook placeholder
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn reward(origin: OriginFor<T>, count: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let amount = T::CxPerTx::get().saturating_mul(count as u128);
            Self::deposit_event(Event::Rewarded { author: who, amount });
            Ok(())
        }
    }
}
