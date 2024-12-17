use candid::{CandidType, Deserialize};
use common::item::spec::SpecValue;
use std::collections::BTreeMap;

type SpecCategoryKey = u8;
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct SpecCategory {
    name: String,
    label_map: BTreeMap<SpecLabelKey, SpecLabel>,
}

impl SpecCategory {
    pub fn builder(name: &str) -> SpecCategoryBuilder {
        SpecCategoryBuilder::new(name)
    }

    pub fn get_label(&self, key: SpecLabelKey) -> Option<&SpecLabel> {
        self.label_map.get(&key)
    }
}

pub struct SpecCategoryBuilder {
    name: String,
    label_map: BTreeMap<SpecLabelKey, SpecLabel>,
}

impl SpecCategoryBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            label_map: BTreeMap::new(),
        }
    }

    pub fn label(mut self, key: SpecLabelKey, label: SpecLabel) -> Self {
        self.label_map.insert(key, label);
        self
    }

    pub fn build(self) -> SpecCategory {
        SpecCategory {
            name: self.name,
            label_map: self.label_map,
        }
    }
}

type SpecLabelKey = u8;
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct SpecLabel {
    name: String,
    value_map: BTreeMap<SpecValueKey, SpecValue>,
}

impl SpecLabel {
    pub fn builder(name: &str) -> SpecLabelBuilder {
        SpecLabelBuilder::new(name)
    }

    pub fn get_value(&self, key: &SpecValueKey) -> Option<&common::item::spec::SpecValue> {
        self.value_map.get(key)
    }
}

pub struct SpecLabelBuilder {
    name: String,
    value_map: BTreeMap<SpecValueKey, SpecValue>,
}

impl SpecLabelBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value_map: BTreeMap::new(),
        }
    }

    pub fn value(mut self, key: SpecValueKey, value: Vec<&str>) -> Self {
        let value = value.iter().map(|v| v.to_string()).collect();

        self.value_map.insert(key, value);
        self
    }

    pub fn build(self) -> SpecLabel {
        SpecLabel {
            name: self.name,
            value_map: self.value_map,
        }
    }
}

type SpecValueKey = u8;

pub type SpecIndexKey = u8;

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct SpecKey {
    category_key: SpecCategoryKey,
    label_vec: Vec<SpecKeyLabel>,
}

impl SpecKey {
    pub fn builder(category_key: SpecCategoryKey) -> SpecKeyBuilder {
        SpecKeyBuilder::new(category_key)
    }
}

pub struct SpecKeyBuilder {
    category_key: SpecCategoryKey,
    label_vec: Vec<SpecKeyLabel>,
}

impl SpecKeyBuilder {
    pub fn new(category_key: SpecCategoryKey) -> Self {
        Self {
            category_key,
            label_vec: Vec::new(),
        }
    }

    pub fn label(mut self, label_key: SpecLabelKey, value_key: SpecValueKey) -> Self {
        self.label_vec.push(SpecKeyLabel::new(label_key, value_key));
        self
    }

    pub fn build(self) -> SpecKey {
        SpecKey {
            category_key: self.category_key,
            label_vec: self.label_vec,
        }
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct SpecKeyLabel {
    label_key: SpecLabelKey,
    value_key: SpecValueKey,
}

impl SpecKeyLabel {
    pub fn new(label_key: SpecLabelKey, value_key: SpecValueKey) -> Self {
        Self {
            label_key,
            value_key,
        }
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct ItemSpecsV1 {
    // Actual data of specs
    pub map: BTreeMap<SpecCategoryKey, SpecCategory>,
    // Mapping of combination of specs
    pub index_map: BTreeMap<SpecIndexKey, SpecKey>,
}

impl ItemSpecsV1 {
    pub fn builder() -> ItemSpecsV1Builder {
        ItemSpecsV1Builder::default()
    }

    pub fn get_specs(&self, keys: &Vec<SpecIndexKey>) -> Option<common::item::spec::SpecResponse> {
        let mut result = Vec::new();

        for key in keys {
            let mut label_vec = Vec::new();
            let spec_key = self.get_spec_key(key)?;

            let category = self.get_category(spec_key.category_key)?;
            for spec_key_label in &spec_key.label_vec {
                let label = category.get_label(spec_key_label.label_key)?;
                let value = label.get_value(&spec_key_label.value_key)?;
                label_vec.push(common::item::spec::SpecResponseLabel {
                    label_name: label.name.clone(),
                    value: value.clone(),
                });
            }

            result.push(common::item::spec::SpecResponseCategory {
                category_name: category.name.clone(),
                label_vec,
            });
        }

        Some(result)
    }

    fn get_spec_key(&self, key: &SpecIndexKey) -> Option<&SpecKey> {
        self.index_map.get(key)
    }

    fn get_category(&self, key: SpecCategoryKey) -> Option<&SpecCategory> {
        self.map.get(&key)
    }
}

#[derive(Default)]
pub struct ItemSpecsV1Builder {
    pub map: BTreeMap<SpecCategoryKey, SpecCategory>,
    pub index_map: BTreeMap<SpecIndexKey, SpecKey>,
}

impl ItemSpecsV1Builder {
    /// Add a new spec category to the builder.
    /// This will be used to store the actual data of the specs.
    ///
    /// The key is used to reference the category.
    ///
    /// The upper is the actual data of the category.
    /// It contains the name of the category and the mapping of the labels.
    pub fn spec(mut self, key: SpecCategoryKey, upper: SpecCategory) -> Self {
        self.map.insert(key, upper);
        self
    }

    /// Add a new spec index to the builder.
    /// This will be used to store the mapping of the combination of specs.
    ///
    /// The key is used to reference the index.
    ///
    /// The index is the actual data of the index.
    /// It contains the category key and the mapping of the labels.
    pub fn index(mut self, key: SpecIndexKey, index: SpecKey) -> Self {
        self.index_map.insert(key, index);
        self
    }

    pub fn build(self) -> ItemSpecsV1 {
        ItemSpecsV1 {
            map: self.map,
            index_map: self.index_map,
        }
    }
}
