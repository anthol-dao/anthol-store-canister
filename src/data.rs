use crate::STORE_DATA;
use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{cell::ValueError, storable::Bound, Storable};
use common::store::{StoreId, StoreName};
use std::borrow::Cow;

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum StoreData {
    None,
    V1(StoreDataV1),
}

#[derive(CandidType, Deserialize, Debug, Clone, Default)]
pub struct StoreDataV1 {
    pub id: StoreId,
    pub name: StoreName,
}

impl Storable for StoreData {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub(crate) fn update_store_data(id: StoreId, name: StoreName) -> Result<StoreData, ValueError> {
    let mut res: Result<StoreData, ValueError> = Ok(StoreData::None);

    STORE_DATA.with_borrow_mut(|store_data| {
        res = store_data.set(StoreData::V1(StoreDataV1 { id, name }))
    });

    res
}
