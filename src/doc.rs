#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId(usize);

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ItemId {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

impl std::str::FromStr for ItemId {
    type Err = nojson::JsonParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        nojson::RawJson::parse(s)?.value().try_into()
    }
}

impl nojson::DisplayJson for ItemId {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct CrateItems(std::collections::HashMap<ItemId, crate::json::JsonValueIndex>);

impl CrateItems {
    pub fn get_item<'a>(
        &self,
        json: &'a nojson::RawJsonOwned,
        item_id: ItemId,
    ) -> Option<nojson::RawJsonValue<'a, 'a>> {
        let i = self.0.get(&item_id)?; // If the item belongs to an external crate, this will return None
        json.get_value_by_index(i.get()) // This always returns Some(_) (unless `json` is incorrect)
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for CrateItems {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

#[derive(Debug)]
pub struct CrateDoc {
    pub path: std::path::PathBuf,
    pub json: nojson::RawJsonOwned,
    pub crate_name: String,
    pub items: CrateItems,
    pub root_module_id: ItemId,
}

impl CrateDoc {
    pub fn parse(path: std::path::PathBuf, text: &str) -> Result<Self, nojson::JsonParseError> {
        let json = nojson::RawJsonOwned::parse(text)?;
        let value = json.value();
        let root_module_id: ItemId = value.to_member("root")?.required()?.try_into()?;
        let items: CrateItems = value.to_member("index")?.required()?.try_into()?;
        todo!()
    }
}
