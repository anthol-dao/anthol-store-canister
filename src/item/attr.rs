use super::{image::ImageVecKey, spec::SpecIndexKey};
use candid::{CandidType, Deserialize};
use common::{
    item::{
        attr::{AttrIndexResponse, AttrIndexesResponse, AttrKey, AttrKeys, AttrType, Stock},
        MediaDataWithCaption,
    },
    unit::{Currency, Price},
};
use std::collections::BTreeMap;

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct AttrSpecificData {
    pub stock: Stock,
    pub price: BTreeMap<Currency, Price>,
    pub image_vec_key: ImageVecKey,
    pub spec_keys: Vec<SpecIndexKey>,
    pub sale: (),
}

impl AttrSpecificData {
    pub fn builder() -> AttrSpecificDataBuilder {
        AttrSpecificDataBuilder::default()
    }
}

#[derive(Default)]
pub struct AttrSpecificDataBuilder {
    stock: Stock,
    price: BTreeMap<Currency, Price>,
    image_vec_key: ImageVecKey,
    spec_keys: Vec<SpecIndexKey>,
    sale: (),
}

impl AttrSpecificDataBuilder {
    pub fn stock(mut self, stock: Stock) -> Self {
        self.stock = stock;
        self
    }

    pub fn price(mut self, currency: Currency, price: f64) -> Self {
        self.price.insert(currency, Price::new(price));
        self
    }

    pub fn image_vec_key(mut self, image_vec_key: ImageVecKey) -> Self {
        self.image_vec_key = image_vec_key;
        self
    }

    pub fn spec_key(mut self, spec_keys: SpecIndexKey) -> Self {
        self.spec_keys.push(spec_keys);
        self
    }

    pub fn build(self) -> AttrSpecificData {
        AttrSpecificData {
            stock: self.stock,
            price: self.price,
            image_vec_key: self.image_vec_key,
            spec_keys: self.spec_keys,
            sale: self.sale,
        }
    }
}

#[derive(CandidType, Deserialize, Debug, Clone, Default)]
pub struct AttrSpecificDataResponse {
    pub stock: Stock,
    pub price: Price,
    pub image_vec: Vec<MediaDataWithCaption>,
    pub specs: Option<common::item::spec::SpecResponse>,
    pub sale: (),
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct AttrCoreSpecificDataResponse {
    pub stock: Stock,
    pub price: Price,
    pub image: MediaDataWithCaption,
    pub sale: (),
}

#[derive(CandidType, Deserialize, Debug, Clone, Default)]
pub struct AttrIndexes([Option<AttrIndex>; 4]);

impl AttrIndexes {
    pub fn builder() -> AttrIndexesBuilder {
        AttrIndexesBuilder::default()
    }
}

#[derive(Default)]
pub struct AttrIndexesBuilder([Option<AttrIndex>; 4]);

impl AttrIndexesBuilder {
    pub fn attr(mut self, index: usize, attr_index: AttrIndex) -> Self {
        self.0[index] = Some(attr_index);
        self
    }

    pub fn build(self) -> AttrIndexes {
        AttrIndexes(self.0)
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct AttrIndex {
    pub name: String,
    pub map: AttrIndexMap,
}

impl AttrIndex {
    pub fn builder(name: &str) -> AttrIndexBuilder {
        AttrIndexBuilder::new(name)
    }
}

pub struct AttrIndexBuilder {
    pub name: String,
    pub map: AttrIndexMap,
}

impl AttrIndexBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            map: BTreeMap::new(),
        }
    }

    pub fn label(mut self, key: AttrKey, label: AttrType) -> Self {
        self.map.insert(key, label);
        self
    }

    pub fn build(self) -> AttrIndex {
        AttrIndex {
            name: self.name,
            map: self.map,
        }
    }
}

type AttrIndexMap = BTreeMap<AttrKey, AttrType>;

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct ItemAttrsV1 {
    pub map: BTreeMap<AttrKeys, AttrSpecificData>,
    pub indexes: AttrIndexes,
}

impl ItemAttrsV1 {
    pub fn builder() -> ItemAttrsV1Builder {
        ItemAttrsV1Builder::default()
    }

    pub fn get_attrs(&self, keys: &AttrKeys) -> Option<AttrSpecificData> {
        let data = self.map.get(keys)?.clone();
        Some(data)
    }

    pub fn get_is_in_stock(&self, attr_keys: &AttrKeys) -> Option<bool> {
        let attr_data = self.map.get(attr_keys)?;
        Some(attr_data.stock > 0)
    }

    pub fn get_attrs_indexes_result(&self) -> AttrIndexesResponse {
        let mut indexes: [Option<AttrIndexResponse>; 4] = [None, None, None, None];

        self.indexes.0.iter().enumerate().for_each(|(i, index)| {
            if let Some(index) = index {
                let vec: Vec<AttrType> = index.map.values().cloned().collect();

                indexes[i] = Some(AttrIndexResponse {
                    name: index.name.clone(),
                    value: vec,
                });
            }
        });

        indexes
    }

    pub fn get_attrs_index_values(&self, attr_keys: AttrKeys) -> Vec<Option<String>> {
        let mut values = Vec::new();

        for (i, index) in self.indexes.0.iter().enumerate() {
            if let Some(index) = index {
                let value = index.map.get(&attr_keys.0[i]).map(|v| match v {
                    AttrType::Text(t) => t.clone(),
                    AttrType::Color(c) => c.name.clone(),
                });
                values.push(value);
            }
        }

        values
    }
}

#[derive(Default)]
pub struct ItemAttrsV1Builder {
    pub map: BTreeMap<AttrKeys, AttrSpecificData>,
    pub indexes: AttrIndexes,
}

impl ItemAttrsV1Builder {
    pub fn attr(mut self, keys: AttrKeys, data: AttrSpecificData) -> Self {
        self.map.insert(keys, data);
        self
    }

    pub fn indexes(mut self, indexes: AttrIndexes) -> Self {
        self.indexes = indexes;
        self
    }

    pub fn build(self) -> ItemAttrsV1 {
        ItemAttrsV1 {
            map: self.map,
            indexes: self.indexes,
        }
    }
}
