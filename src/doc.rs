use crate::json::JsonValueIndex;

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

#[derive(Debug, Clone)]
pub struct PublicItem {
    pub path: ItemPath,
    pub kind: &'static str,
    pub index: JsonValueIndex,
}

#[derive(Debug, Clone)]
pub struct ItemPath(Vec<String>);

impl ItemPath {
    pub fn crate_name(&self) -> &str {
        &self.0[0]
    }

    pub fn name(&self) -> &str {
        self.0.last().expect("bug")
    }
}

#[derive(Debug)]
pub struct CrateItems(std::collections::HashMap<ItemId, JsonValueIndex>);

impl CrateItems {
    fn get<'a>(
        &self,
        json: &'a nojson::RawJsonOwned,
        item_id_value: nojson::RawJsonValue<'a, 'a>,
    ) -> Result<nojson::RawJsonValue<'a, 'a>, nojson::JsonParseError> {
        let item_id: ItemId = item_id_value.try_into()?;
        let i = self
            .0
            .get(&item_id)
            .ok_or_else(|| item_id_value.invalid("item does not exist in this crate"))?;
        Ok(json.get_value_by_index(i.get()).expect("bug"))
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
    pub root_module_index: JsonValueIndex,
    pub public_items: Vec<PublicItem>,
}

impl CrateDoc {
    pub fn parse(path: std::path::PathBuf, text: &str) -> Result<Self, nojson::JsonParseError> {
        let json = nojson::RawJsonOwned::parse(text)?;
        let value = json.value();
        let root_module_id_value = value.to_member("root")?.required()?;
        let items: CrateItems = value.to_member("index")?.required()?.try_into()?;
        let root_module_value = items.get(&json, root_module_id_value)?;
        let crate_name = root_module_value
            .to_member("name")?
            .required()?
            .try_into()?;
        let root_module_index = root_module_value.try_into()?;
        let mut this = Self {
            path,
            json,
            crate_name,
            items,
            root_module_index,
            public_items: Vec::new(),
        };
        this.collect_public_items()?;
        Ok(this)
    }

    fn collect_public_items(&mut self) -> Result<(), nojson::JsonParseError> {
        let mut path = ItemPath(Vec::new());
        self.visit_item(&mut path, self.root_module_index)?;
        Ok(())
    }

    fn visit_item(
        &mut self,
        path: &mut ItemPath,
        item_index: JsonValueIndex,
    ) -> Result<(), nojson::JsonParseError> {
        let item = self.json.get_value_by_index(item_index.0).expect("bug");

        // Recursively visit modules; skip non-public items
        let (kind, inner) = item
            .to_member("inner")?
            .required()?
            .to_object()?
            .next()
            .ok_or_else(|| item.invalid("empty inner"))?;

        let kind_str = kind.to_unquoted_string_str()?;

        if kind_str == "module" {
            self.visit_module(path, item_index)?;
        }

        Ok(())
    }

    fn visit_module(
        &mut self,
        path: &mut ItemPath,
        module_index: JsonValueIndex,
    ) -> Result<(), nojson::JsonParseError> {
        let module = self.json.get_value_by_index(module_index.0).expect("bug");

        // Check if module is public
        let visibility = module
            .to_member("visibility")?
            .required()?
            .to_unquoted_string_str()?;

        if visibility != "public" {
            return Ok(());
        }

        // Get module name and add to path
        let name: String = module.to_member("name")?.required()?.try_into()?;
        path.0.push(name);

        // Get the inner module data and iterate through items
        let inner = module
            .to_member("inner")?
            .required()?
            .to_member("module")?
            .required()?;

        for item_id_value in inner.to_member("items")?.required()?.to_array()? {
            let item_id: ItemId = item_id_value.try_into()?;
            let item_index = self
                .items
                .0
                .get(&item_id)
                .copied()
                .ok_or_else(|| item_id_value.invalid("item not found"))?;
            self.visit_item(path, item_id, item_index)?;
        }

        path.0.pop();
        Ok(())
    }
}
