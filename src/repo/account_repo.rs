use std::collections::HashMap;

use async_trait::async_trait;

use crate::{app::{AccountRepository}, LedgerResult, Account};

pub struct LedgerAccountRepository {
    accounts: HashMap<u16, Account>
}

#[async_trait]
impl AccountRepository for LedgerAccountRepository {
    async fn create_account(&self, client_id: u16) -> LedgerResult<Account>{
        todo!()
    }
    async fn exists(&self, client_id: u16) -> LedgerResult<bool>{
        todo!()
    }
    async fn hold(&self, client_id: u16, amount: f32) -> LedgerResult<()>{
        todo!()
    }
    async fn release(&self, client_id: u16, amount: f32) -> LedgerResult<()>{
        todo!()
    }
    async fn deposit(&self, client_id: u16, amount: f32) -> LedgerResult<()>{
        todo!()
    }
    async fn withdraw(&self, client_id: u16, amount: f32) -> LedgerResult<()>{
        todo!()
    }
    async fn lock(&self, client_id: u16) -> LedgerResult<()>{
        todo!()
    }
    async fn is_locked(&self, client_id: u16) -> LedgerResult<bool>{
        todo!()
    }
    async fn dump_accounts(&self) -> LedgerResult<Vec<Account>>{
        todo!()
    }
}