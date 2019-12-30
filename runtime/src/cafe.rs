use parity_codec::{Decode, Encode};
use rstd::vec::Vec;
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageMap, StorageValue,
};
use system::{self, ensure_signed};

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Cafe<AccountId> {
    owner: AccountId,
    waiters: Vec<AccountId>
}

pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId
    {
        EmissionProject(AccountId, u64),
        Transferred(AccountId, AccountId, u64),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as CafeStorage {
        TotalSupply get(total_supply): u64;
        CafeOf get(cafe_of): map T::AccountId => Cafe<T::AccountId>;
        AccountOf get(account_of): map T::AccountId => u64;
  }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        fn add_cafe(
            origin,
            cafe_account: T::AccountId,
            waiters: Vec<T::AccountId>
        ) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(!<CafeOf<T>>::exists(cafe_account.clone()), "cafe already exists.");
            let new_cafe = Cafe {
                owner: sender,
                waiters: waiters
            };
            <CafeOf<T>>::insert(cafe_account.clone(), new_cafe.clone());
            Ok(())
        }

        fn emission_cafe(
            origin,
            cafe_account: T::AccountId,
            value: u64
        ) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(<CafeOf<T>>::exists(cafe_account.clone()), "cafe does not exists.");
            let cafe = Self::cafe_of(cafe_account.clone());
            ensure!(sender == cafe.owner, "you are not owner.");
            let mut account = Self::account_of(cafe_account.clone());
            account = account.checked_add(value.clone()).ok_or("overflow in calculating balance")?;
            <AccountOf<T>>::insert(cafe_account.clone(), account.clone());
            Self::_update_total(0, value)
        }

        fn remove_cafe(
            origin,
            cafe_account: T::AccountId 
        ) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(<CafeOf<T>>::exists(cafe_account.clone()), "cafe does not exists.");
            let cafe = Self::cafe_of(cafe_account.clone());
            ensure!(sender == cafe.owner, "you are not owner.");
            let account = Self::account_of(cafe_account.clone());
            <AccountOf<T>>::remove(cafe_account.clone());
            <CafeOf<T>>::remove(cafe_account.clone());
            Self::_update_total(account, 0)
        }

        fn add_waiter(
        	origin,
        	cafe_account: T::AccountId,
        	waiter: T::AccountId
       	) -> Result {
       		let sender = ensure_signed(origin)?;
            ensure!(<CafeOf<T>>::exists(cafe_account.clone()), "cafe does not exists.");
            let mut cafe = Self::cafe_of(cafe_account.clone());
            ensure!(sender == cafe.owner, "you are not owner.");
        	cafe.waiters.push(waiter.clone());
        	<CafeOf<T>>::insert(cafe_account.clone(), cafe.clone());
        	Ok(())
       }

       fn delete_waiter(
        	origin,
        	cafe_account: T::AccountId,
        	waiter: T::AccountId
       	) -> Result {
       		let sender = ensure_signed(origin)?;
            ensure!(<CafeOf<T>>::exists(cafe_account.clone()), "cafe does not exists.");
            let mut cafe = Self::cafe_of(cafe_account.clone());
            ensure!(sender == cafe.owner.clone(), "you are not owner.");
            let index =  cafe.waiters.iter().position(|x| *x == waiter.clone()).unwrap();
            cafe.waiters.remove(index);
        	<CafeOf<T>>::insert(cafe_account.clone(), cafe.clone());
        	Ok(())
       }

       fn add_bonus(
            origin,
            cafe_account: T::AccountId,
            to: T::AccountId,
            value: u64
        ) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(<CafeOf<T>>::exists(cafe_account.clone()), "cafe does not exists.");
            let cafe = Self::cafe_of(cafe_account.clone());
            let waiter = cafe.waiters.iter().find(|&s| *s == sender.clone());
            ensure!(waiter != None, "you are not waiter.");
            Self::_transfer_from(cafe_account, to, value)
        }

        fn spent_bonus(
            origin,
            to: T::AccountId,
            value: u64
        ) -> Result {
            let sender = ensure_signed(origin)?;
            Self::_transfer_from(sender, to, value)
        }
    }
}

impl<T: Trait> Module<T> {

	fn _update_total(
		value: u64, 
		new_value: u64
	) -> Result {
        let mut total = Self::total_supply();
        total = total
            .checked_sub(value)
            .ok_or("overflow in calculating balance")?;
        total = total
            .checked_add(new_value)
            .ok_or("overflow in calculating balance")?;
        <TotalSupply<T>>::put(total);
        Ok(())
    }

    fn _transfer_from(
    	from: T::AccountId, 
    	to: T::AccountId, 
    	value: u64
    ) -> Result {
        let sender_balance = Self::account_of(from.clone());
        ensure!(sender_balance >= value, "Not enough balance.");

        let updated_from_balance = sender_balance
            .checked_sub(value)
            .ok_or("overflow in calculating balance")?;
        let receiver_balance = Self::account_of(to.clone());
        let updated_to_balance = receiver_balance
            .checked_add(value)
            .ok_or("overflow in calculating balance")?;

        <AccountOf<T>>::insert(from.clone(), updated_from_balance);
        <AccountOf<T>>::insert(to.clone(), updated_to_balance);

        Self::deposit_event(RawEvent::Transferred(from, to, value));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Blake2Hasher, H256};
    use runtime_io::{with_externalities, TestExternalities};
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_noop, assert_ok, impl_outer_origin};

    impl_outer_origin! {
        pub enum Origin for CafeTest {}
    }

    #[derive(Clone, Eq, PartialEq)]
    pub struct CafeTest;

    impl system::Trait for CafeTest {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }

    impl balances::Trait for CafeTest {
        type Balance = u64;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type TransactionPayment = ();
        type TransferPayment = ();
        type DustRemoval = ();
    }

    impl super::Trait for CafeTest {
        type Event = ();
    }

    type Cafe = super::Module<CafeTest>;

    fn build_ext() -> TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<CafeTest>::default()
            .build_storage()
            .unwrap()
            .0;
        t.extend(
            balances::GenesisConfig::<CafeTest>::default()
                .build_storage()
                .unwrap()
                .0,
        );
        t.into()
    }

    #[test]
    fn add_cafe() {
        with_externalities(&mut build_ext(), || {
        	let owner = 1000;
            let cafe_account = 1001;
            let waiters = [102,103,104];
            assert_ok!(Cafe::add_cafe(Origin::signed(owner), cafe_account, waiters.to_vec()));
            assert_noop!(Cafe::add_cafe(Origin::signed(owner), cafe_account, waiters.to_vec()), "cafe already exists.");


            let cafe = Cafe::cafe_of(cafe_account);
            assert_eq!(cafe.owner, owner);
            assert_eq!(cafe.waiters, waiters);
        })
    }

    #[test]
    fn emission_cafe() {
        with_externalities(&mut build_ext(), || {
        	let owner = 1000;
            let cafe_account = 1001;
            let waiters = [102,103,104];
            assert_noop!(Cafe::emission_cafe(Origin::signed(owner), cafe_account, 1000), "cafe does not exists.");
            assert_ok!(Cafe::add_cafe(Origin::signed(owner), cafe_account, waiters.to_vec()));
            assert_noop!(Cafe::emission_cafe(Origin::signed(1101), cafe_account, 1000), "you are not owner.");
            assert_ok!(Cafe::emission_cafe(Origin::signed(owner), cafe_account, 1000));

            let cafe_balance = Cafe::account_of(cafe_account);
            assert_eq!(cafe_balance, 1000);
        })
    }

    #[test]
    fn remove_cafe() {
        with_externalities(&mut build_ext(), || {
        	let owner = 1000;
            let cafe_account = 1001;
            let waiters = [102,103,104];
            assert_noop!(Cafe::remove_cafe(Origin::signed(owner), cafe_account), "cafe does not exists.");
            assert_ok!(Cafe::add_cafe(Origin::signed(owner), cafe_account, waiters.to_vec()));
            assert_ok!(Cafe::emission_cafe(Origin::signed(owner), cafe_account, 1000));
            assert_noop!(Cafe::remove_cafe(Origin::signed(1101), cafe_account), "you are not owner.");
            assert_ok!(Cafe::remove_cafe(Origin::signed(owner), cafe_account));

            let cafe_balance = Cafe::account_of(cafe_account);
            assert_eq!(cafe_balance, 0);
        })
    }

    #[test]
    fn add_waiter() {
        with_externalities(&mut build_ext(), || {
        	let owner = 1000;
            let cafe_account = 1001;
            let waiters = [];
            let new_waiter = 1101;
            assert_noop!(Cafe::remove_cafe(Origin::signed(owner), cafe_account), "cafe does not exists.");
            assert_ok!(Cafe::add_cafe(Origin::signed(owner), cafe_account, waiters.to_vec()));

            let cafe = Cafe::cafe_of(cafe_account);
            assert_eq!(cafe.owner, owner);
            assert_eq!(cafe.waiters, waiters);

            assert_noop!(Cafe::add_waiter(Origin::signed(new_waiter), cafe_account, new_waiter), "you are not owner.");
            assert_ok!(Cafe::add_waiter(Origin::signed(owner), cafe_account, new_waiter));

            let cafe = Cafe::cafe_of(cafe_account);
            assert_eq!(cafe.owner, owner);
            assert_eq!(cafe.waiters, [new_waiter]);
        })
    }

    #[test]
    fn delete_waiter() {
        with_externalities(&mut build_ext(), || {
        	let owner = 1000;
            let cafe_account = 1001;
            let waiters = [100,200];
            let old_waiter = 100;
            assert_noop!(Cafe::remove_cafe(Origin::signed(owner), cafe_account), "cafe does not exists.");
            assert_ok!(Cafe::add_cafe(Origin::signed(owner), cafe_account, waiters.to_vec()));

            let cafe = Cafe::cafe_of(cafe_account);
            assert_eq!(cafe.owner, owner);
            assert_eq!(cafe.waiters, waiters);

            assert_noop!(Cafe::delete_waiter(Origin::signed(old_waiter), cafe_account, old_waiter), "you are not owner.");
            assert_ok!(Cafe::delete_waiter(Origin::signed(owner), cafe_account, old_waiter));

            let cafe = Cafe::cafe_of(cafe_account);
            assert_eq!(cafe.owner, owner);
            assert_eq!(cafe.waiters, [200]);
        })
    }

    #[test]
    fn add_bonus() {
        with_externalities(&mut build_ext(), || {
        	let owner = 1000;
            let cafe_account = 1001;
            let waiters = [100,200];
            let to = 1002;
            assert_noop!(Cafe::add_bonus(Origin::signed(100), cafe_account, to, 10), "cafe does not exists.");
            assert_ok!(Cafe::add_cafe(Origin::signed(owner), cafe_account, waiters.to_vec()));
            assert_ok!(Cafe::emission_cafe(Origin::signed(owner), cafe_account, 1000));

            let cafe = Cafe::cafe_of(cafe_account);
            assert_eq!(cafe.owner, owner);
            assert_eq!(cafe.waiters, waiters);

            let cafe_balance = Cafe::account_of(cafe_account);
            assert_eq!(cafe_balance, 1000);

            assert_noop!(Cafe::add_bonus(Origin::signed(owner), cafe_account, to, 10), "you are not waiter.");
            assert_ok!(Cafe::add_bonus(Origin::signed(100), cafe_account, to, 10));

            let cafe_balance = Cafe::account_of(cafe_account);
            assert_eq!(cafe_balance, 990);
            let to_balance = Cafe::account_of(to);
            assert_eq!(to_balance, 10);
        })
    }

    #[test]
    fn spent_bonus() {
        with_externalities(&mut build_ext(), || {
        	let owner = 1000;
            let cafe_account = 1001;
            let waiters = [100,200];
            let from = 1002;
            let to = 1003;

            assert_ok!(Cafe::add_cafe(Origin::signed(owner), cafe_account, waiters.to_vec()));
            assert_ok!(Cafe::emission_cafe(Origin::signed(owner), cafe_account, 1000));

            let cafe_balance = Cafe::account_of(cafe_account);
            assert_eq!(cafe_balance, 1000);
            assert_ok!(Cafe::add_bonus(Origin::signed(100), cafe_account, from, 100));

            assert_ok!(Cafe::spent_bonus(Origin::signed(from), to, 45));

            let from_balance = Cafe::account_of(from);
            assert_eq!(from_balance, 55);
            let to_balance = Cafe::account_of(to);
            assert_eq!(to_balance, 45);
        })
    }

}