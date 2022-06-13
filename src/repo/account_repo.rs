use std::collections::HashMap;

use async_trait::async_trait;
use futures::lock::Mutex;

use crate::{app::{AccountRepository}, LedgerResult, Account, LedgerError};

pub struct LedgerAccountRepository {
    accounts: Mutex<HashMap<u16, Account>>
}

#[async_trait]
impl AccountRepository for LedgerAccountRepository {
    async fn get_or_create_account(&mut self, client_id: u16) -> LedgerResult<Account>{
        let mut repo = self.accounts.lock().await;
        let a = match repo.get(&client_id) {
            Some(a) => a.clone(),
            None => {
                let a = Account::new(client_id);
                repo.insert(client_id, a);
                a
            },
        };

        Ok(a)
    }
    async fn hold(&mut self, client_id: u16, amount: f32) -> LedgerResult<()>{
        let mut a = self.get_account(client_id).await?;
        if a.is_locked() {
            return account_err("account is locked");
        }

        a.hold(amount);
        self.update_account(client_id, a).await
    }
    async fn release(&mut self, client_id: u16, amount: f32) -> LedgerResult<()>{
        let mut a = self.get_account(client_id).await?;
        if a.is_locked() {
            return account_err("account is locked");
        }

        a.release(amount);
        self.update_account(client_id, a).await
    }
    async fn deposit(&mut self, client_id: u16, amount: f32) -> LedgerResult<()>{
        let mut a = self.get_account(client_id).await?;
        if a.is_locked() {
            return account_err("account is locked");
        }

        a.deposit(amount);
        self.update_account(client_id, a).await
    }
    async fn withdraw(&mut self, client_id: u16, amount: f32) -> LedgerResult<()>{
        let mut a = self.get_account(client_id).await?;
        if a.is_locked() {
            return account_err("account is locked");
        }

        a.withdraw(amount);
        self.update_account(client_id, a).await
    }
    async fn withdraw_and_lock(&mut self, client_id: u16, amount: f32) -> LedgerResult<()>{
        let mut a = self.get_account(client_id).await?;
        if a.is_locked() {
            return account_err("account is locked");
        }

        a.withdraw_and_lock(amount);
        self.update_account(client_id, a).await
    }
    async fn dump_accounts(&self) -> LedgerResult<Vec<Account>>{
        todo!()
    }
}

impl LedgerAccountRepository {
    async fn get_account(&self, client_id: u16) -> LedgerResult<Account> {
        let a = self.accounts.lock().await.get(&client_id)
            .ok_or(LedgerError::doesnt_exist("account does not exist"))?.clone();

        Ok(a)
    }
    async fn update_account(&mut self, client_id: u16, account: Account) -> LedgerResult<()> {
        self.accounts.lock().await.insert(client_id, account);

        Ok(())
    }
}

fn account_err(msg: &str) -> Result<(), LedgerError> {
    Err(LedgerError::repository_error(msg))
}