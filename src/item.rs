use crate::data::StoreData;

use super::{ITEMS, ITEMS_IN_ID, STORE_DATA};
use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};
pub use shared::{
    item::{
        attr::{AttrKeys, AttrStatusResponse, AttrStatusesResponse},
        ItemCoreDataRequest, ItemId, ItemKey, ItemName, ItemPageFromStoreErrorCode,
        ItemPageRequestToStoreCanister, ItemPageResponseFromStoreCanister,
        ItemPageStaticDataFromStoreCanister, Tag,
    },
    unit::Currency,
};
use std::borrow::Cow;

pub mod attr;
use attr::{AttrCoreSpecificDataResponse, AttrSpecificDataResponse, ItemAttrsV1};
pub mod image;
use image::ItemImagesV1;
pub mod spec;
use spec::ItemSpecsV1;

nest! {
    /// Represents an item with its associated data.
    #[derive(CandidType, Deserialize, Debug, Clone)]
    pub struct Item {
        pub id: ItemId,
        pub name: ItemName,
        #>[derive(CandidType, Deserialize, Debug, Clone)]
        pub version: pub enum ItemVersion {
            V1 {
                descriptions: Vec<String>,
                tags: Vec<Tag>,
                images: ItemImagesV1,
                specs: ItemSpecsV1,
                attrs: ItemAttrsV1,
            }
        },
    }
}

impl Storable for Item {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Item {
    /// Retrieves specific attribute data for an item based on provided keys and currency.
    pub fn get_attr_data(
        &self,
        attr_keys: &AttrKeys,
        currency: &Currency,
    ) -> Option<AttrSpecificDataResponse> {
        match &self.version {
            ItemVersion::V1 {
                attrs,
                images,
                specs,
                ..
            } => {
                let attr_data = attrs.map.get(attr_keys)?;
                let price = *attr_data.price.get(currency)?;
                let image_vec = images.get_index_vec(&attr_data.image_vec_key)?;
                let specs = specs.get_specs(&attr_data.spec_keys);

                let res = AttrSpecificDataResponse {
                    stock: attr_data.stock,
                    price,
                    image_vec,
                    specs,
                    sale: attr_data.sale,
                };

                Some(res)
            }
        }
    }

    pub fn get_attr_statuses(&self, attr_keys: &AttrKeys) -> AttrStatusesResponse {
        let mut result = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];

        match &self.version {
            ItemVersion::V1 { attrs, .. } => {
                let indexes = attrs.get_attrs_indexes_result();

                indexes
                    .iter()
                    .enumerate()
                    .for_each(|(i, index)| match index {
                        Some(index) => {
                            let mut each_result = Vec::new();

                            index.value.iter().enumerate().for_each(|(j, _)| {
                                let attr_keys = attr_keys.replace(i, &(j as u8)).unwrap();
                                match attrs.map.get(&attr_keys) {
                                    Some(attr_data) => {
                                        let is_in_stock = attr_data.stock > 0;
                                        each_result.push(Some(AttrStatusResponse { is_in_stock }));
                                    }
                                    None => {
                                        each_result.push(None);
                                    }
                                }
                            });

                            result[i] = each_result;
                        }
                        None => {}
                    });
            }
        }

        result
    }

    /// Retrieves core attribute data for an item, which includes stock, price, base image, and sale status.
    pub fn get_attr_core_data(
        &self,
        attr_keys: &AttrKeys,
        currency: &Currency,
    ) -> Option<AttrCoreSpecificDataResponse> {
        match &self.version {
            ItemVersion::V1 { attrs, images, .. } => {
                let attr_data = attrs.map.get(attr_keys)?;
                let price = *attr_data.price.get(currency)?;
                let image = images.get_base_image(&attr_data.image_vec_key)?.clone();

                let res = AttrCoreSpecificDataResponse {
                    stock: attr_data.stock,
                    price,
                    image,
                    sale: attr_data.sale,
                };

                Some(res)
            }
        }
    }
}

/// Fetches the page information for a specific item.
pub(crate) fn get_item_page_data(
    arg: &ItemPageRequestToStoreCanister,
) -> Result<ItemPageResponseFromStoreCanister, (ItemPageFromStoreErrorCode, String)> {
    let mut static_data: Option<ItemPageStaticDataFromStoreCanister> = None;

    let item_key = match ITEMS_IN_ID.with(|p| p.borrow().get(&arg.item_id)) {
        Some(item) => item,
        None => {
            return Err((
                ItemPageFromStoreErrorCode::ItemNotFound,
                format!("Item with id {} not found in the store", arg.item_id),
            ));
        }
    };

    let item = match ITEMS.with(|p| p.borrow().get(&item_key)) {
        Some(item) => item,
        None => {
            return Err((
                ItemPageFromStoreErrorCode::ItemNotFound,
                format!("Item with key {} not found in the store", item_key),
            ));
        }
    };

    let item_name = item.name.clone();

    let store_name = STORE_DATA.with(|p| match p.borrow().get() {
        StoreData::V1(store_data) => store_data.name.clone(),
        StoreData::None => "".try_into().unwrap(),
    });

    if arg.attr.changed_key_index.is_none() {
        match item.version {
            ItemVersion::V1 {
                ref attrs,
                ref descriptions,
                ref tags,
                ..
            } => {
                let attrs = attrs.get_attrs_indexes_result();

                static_data = Some(ItemPageStaticDataFromStoreCanister {
                    item_name,
                    descriptions: descriptions.clone(),
                    tags: tags.clone(),
                    attrs,
                    store_name,
                });
            }
        }
    }

    let mut attr_status = item.get_attr_statuses(&arg.attr.keys);

    let (attr_data, fallback_attr) = get_attr_data_or_fallback(&item, arg, &attr_status);

    if fallback_attr.is_some() {
        attr_status = item.get_attr_statuses(&fallback_attr.unwrap());
    }

    let attr_data = match attr_data {
        Some(attr_data) => attr_data,
        None => {
            return Err((
                ItemPageFromStoreErrorCode::NoAvailableAttr,
                format!(
                    "Attribute with keys {:?} of item with id {} not found in the store",
                    arg.attr.keys, arg.item_id
                ),
            ));
        }
    };

    let res = ItemPageResponseFromStoreCanister {
        static_data,
        price: attr_data.price,
        images: attr_data.image_vec,
        stock: attr_data.stock,
        attr_status,
        specs: attr_data.specs,
        fallback_attr,
    };

    Ok(res)
}

pub(crate) fn get_attr_data_or_fallback(
    item: &Item,
    arg: &ItemPageRequestToStoreCanister,
    attr_status: &AttrStatusesResponse,
) -> (Option<AttrSpecificDataResponse>, Option<AttrKeys>) {
    if let Some(attr_data) = item.get_attr_data(&arg.attr.keys, &arg.currency) {
        return (Some(attr_data), None);
    }

    if let Some(changed_key) = arg.attr.changed_key_index {
        for (i, attr_status_one) in attr_status.iter().enumerate() {
            if i == changed_key as usize {
                continue;
            }
            for (j, status) in attr_status_one.iter().enumerate() {
                if status.is_some() {
                    let attr_keys = match arg.attr.keys.replace(i, &(j as u8)) {
                        Ok(attr_keys) => attr_keys,
                        Err(_) => continue,
                    };
                    let attr_data = match item.get_attr_data(&attr_keys, &arg.currency) {
                        Some(attr_data) => attr_data,
                        None => continue,
                    };
                    return (Some(attr_data), Some(attr_keys));
                }
            }
        }
    }

    let attr_keys = AttrKeys::default();
    if let Some(attr_data) = item.get_attr_data(&attr_keys, &arg.currency) {
        return (Some(attr_data), Some(attr_keys));
    }

    (None, None)
}

pub(crate) async fn insert_items(vec: Vec<Item>) -> Vec<(ItemId, ItemKey)> {
    let mut prev_items: Vec<(ItemId, ItemKey)> = Vec::new();
    let mut keys: Vec<ItemKey> = Vec::new();
    let mut ids: Vec<ItemId> = Vec::new();

    let mut i = 0;
    'outer: loop {
        let four_keys = ItemKey::create_four().await.unwrap(); // TODO: handle error
        let mut j = 0;
        while j < 4 {
            keys.push(four_keys[j]);
            j += 1;
            i += 1;

            if i >= vec.len() {
                break 'outer;
            }
        }
    }

    ITEMS.with_borrow_mut(|p| {
        for (i, item) in vec.into_iter().enumerate() {
            ids.push(item.id);
            p.insert(keys[i], item);
        }
    });

    ITEMS_IN_ID.with_borrow_mut(|p| {
        for (i, key) in keys.into_iter().enumerate() {
            if let Some(prev_key) = p.insert(ids[i], key) {
                prev_items.push((ids[i], prev_key));
            };
        }
    });

    prev_items
}
