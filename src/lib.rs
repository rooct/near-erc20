use near_sdk::{
    collections::UnorderedMap, env::predecessor_account_id, json_types::U128, require, AccountId,
    IntoStorageKey,
};

#[derive(Debug)]
pub struct ERC20 {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u128,
    pub balance: UnorderedMap<AccountId, u128>,
    pub allowed: UnorderedMap<AccountId, UnorderedMap<AccountId, u128>>,
}

impl ERC20 {
    pub fn init<B, A>(
        name: String,
        symbol: String,
        decimals: u8,
        total_supply: U128,
        balance_prefix: B,
        allowed_prefix: A,
    ) -> Self
    where
        B: IntoStorageKey,
        A: IntoStorageKey,
    {
        Self {
            name,
            symbol,
            decimals,
            total_supply: total_supply.into(),
            balance: UnorderedMap::new(balance_prefix),
            allowed: UnorderedMap::new(allowed_prefix),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn decimals(&self) -> &u8 {
        &self.decimals
    }

    pub fn total_supply(&self) -> &u128 {
        &self.total_supply
    }

    pub fn balance_of(&self, account_id: AccountId) -> Option<u128> {
        self.balance.get(&account_id)
    }

    pub fn transfer(&mut self, to: AccountId, value: U128) -> bool {
        let user_balance = self.balance_of(predecessor_account_id()).unwrap_or(0u128);
        let value = value.into();
        require!(user_balance >= value);
        self.balance
            .insert(&predecessor_account_id(), &(user_balance - value));

        let mut receiver_balance = self.balance_of(to.clone()).unwrap_or(0u128);
        if let 0 = receiver_balance {
            self.balance.insert(&predecessor_account_id(), &0u128);
            receiver_balance = 0u128;
        }

        self.balance.insert(&to, &(receiver_balance + value));

        true
    }

    pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: U128) -> bool {
        let user_balance = self.balance_of(from.clone()).unwrap();
        let value = value.into();
        require!(user_balance >= value);
        require!(self.allowance(from.clone(), predecessor_account_id()) >= value);
        self.balance.insert(&from, &(user_balance - value)).unwrap();

        let mut receiver_balance = self.balance_of(to.clone()).unwrap_or(0u128);
        if let 0 = receiver_balance {
            self.balance.insert(&predecessor_account_id(), &0u128);
            receiver_balance = 0u128;
        }

        self.balance
            .insert(&to, &(receiver_balance + value))
            .unwrap();

        true
    }

    pub fn approve(&mut self, spender: AccountId, value: U128) {
        let allowance_exist = self.allowed.get(&predecessor_account_id());
        if allowance_exist.is_none() {
            self.allowed.insert(
                &predecessor_account_id(),
                &UnorderedMap::new(near_sdk::env::keccak256(spender.as_bytes())),
            );
        }

        self.allowed
            .get(&predecessor_account_id())
            .unwrap()
            .insert(&spender, &value.into());
    }

    pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
        match self.allowed.get(&owner) {
            Some(r) => r.get(&spender).unwrap_or(0),
            None => 0,
        }
    }

    pub fn mint(&mut self, to: AccountId, value: U128) {
        if self.balance.get(&to).is_none() {
            self.balance.insert(&to, &value.0);
            return;
        }
        let temp = self.balance.get(&to).expect("get failed");
        self.balance.insert(&to, &(value.0 + temp));
    }

    pub fn burn(&mut self, account_id: AccountId, value: U128) {
        require!(value.0 != 0);
        require!(self.balance_of(account_id.clone()).unwrap_or(0u128) >= value.0);
        let temp = self.balance.get(&account_id).expect("get failed");
        self.balance.insert(&account_id, &(temp - value.0));
    }
}
