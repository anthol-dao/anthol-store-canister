use candid::{CandidType, Deserialize};
use common::item::MediaDataWithCaption;
use std::collections::BTreeMap;

type ImageKey = u8;

pub type ImageVecKey = u32;

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct ItemImagesV1 {
    // Actual data of images
    pub map: BTreeMap<ImageKey, MediaDataWithCaption>,
    // Mapping of image groups to keys
    pub index_vec_map: BTreeMap<ImageVecKey, Vec<ImageKey>>,
}

impl ItemImagesV1 {
    pub fn builder() -> ItemImagesV1Builder {
        ItemImagesV1Builder::default()
    }

    pub fn get_base_image(&self, key: &ImageVecKey) -> Option<&MediaDataWithCaption> {
        let image_key = self.index_vec_map.get(key)?[0];
        self.map.get(&image_key)
    }

    pub fn get_index_vec(&self, key: &ImageVecKey) -> Option<Vec<MediaDataWithCaption>> {
        let image_vec = self.index_vec_map.get(key)?;
        let mut images = Vec::new();
        for image_key in image_vec {
            images.push(self.map.get(image_key)?.clone());
        }
        Some(images)
    }
}

#[derive(Default)]
pub struct ItemImagesV1Builder {
    pub map: BTreeMap<ImageKey, MediaDataWithCaption>,
    pub index_vec_map: BTreeMap<ImageVecKey, Vec<ImageKey>>,
}

impl ItemImagesV1Builder {
    pub fn image(&mut self, key: ImageKey, image: MediaDataWithCaption) -> &mut Self {
        self.map.insert(key, image);
        self
    }

    pub fn index_vec(&mut self, key: ImageVecKey, image_vec: Vec<ImageKey>) -> &mut Self {
        self.index_vec_map.insert(key, image_vec);
        self
    }

    pub fn build(&self) -> ItemImagesV1 {
        ItemImagesV1 {
            map: self.map.clone(),
            index_vec_map: self.index_vec_map.clone(),
        }
    }
}
