#[macro_use]
extern crate nestify;

use candid::Principal;
use ic_cdk_macros::*;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, StableCell, StableLog,
};
use shared::{
    item::{
        ItemId, ItemKey, ItemPageFromStoreErrorCode, ItemPageRequestToStoreCanister,
        ItemPageResponseFromStoreCanister,
    },
    store::{StoreId, StoreInitArg, StoreName},
};
use std::cell::RefCell;

pub mod data;
pub mod item;
mod log;

use data::StoreData;
use item::Item;
use log::{LogEntry, LogLevel};

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub(crate) static LOG: RefCell<StableLog<LogEntry, Memory, Memory>> = RefCell::new(
        StableLog::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        ).unwrap()
    );

    pub(crate) static STORE_DATA: RefCell<StableCell<StoreData, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
            StoreData::None,
        ).unwrap()
    );

    pub(crate) static ITEMS: RefCell<StableBTreeMap<ItemKey, Item, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
        )
    );

    pub(crate) static ITEMS_IN_ID: RefCell<StableBTreeMap<ItemId, ItemKey, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))),
        )
    );
}

#[init]
fn init_store(arg: Option<StoreInitArg>) {
    if let Some(arg) = arg {
        let _ = data::update_store_data(arg.id, arg.name);
    }
}

#[update]
fn update_store_data(caller: Principal, id: StoreId, name: StoreName) {
    let _ = data::update_store_data(id, name);
}

#[query]
fn get_item_page_data_from_store(
    caller: Principal,
    arg: ItemPageRequestToStoreCanister,
) -> Result<ItemPageResponseFromStoreCanister, (ItemPageFromStoreErrorCode, String)> {
    let res = crate::item::get_item_page_data(&arg);

    let log_entry = match res.as_ref() {
        Ok(_) => LogEntry::new(LogLevel::Info, Some(caller), "get_item_page_data: Ok", None),
        Err((code, message)) => {
            ic_cdk::println!(
                "get_item_page_data: Err: code: {:?}, message: {}",
                code,
                message
            );
            LogEntry::new(
                LogLevel::Error,
                Some(caller),
                "get_item_page_data: Err",
                Some(format!("code: {:?}, message: {}", code, message).as_str()),
            )
        }
    };

    LOG.with(|log| log.borrow_mut().append(&log_entry).unwrap());

    res
}

#[update]
async fn insert_items_to_store(caller: Principal, vec: Vec<Item>) -> Vec<(ItemId, ItemKey)> {
    crate::item::insert_items(vec).await
}
