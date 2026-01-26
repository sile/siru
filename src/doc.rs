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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Module,
    Use,
    Enum,
    Variant,
    Struct,
    TypeAlias,
    Function,
    Constant,
    Trait,
    AssocType,
    AssocConst,
    Macro,
    Impl,
}

impl ItemKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemKind::Module => "module",
            ItemKind::Use => "use",
            ItemKind::Enum => "enum",
            ItemKind::Variant => "variant",
            ItemKind::Struct => "struct",
            ItemKind::TypeAlias => "type_alias",
            ItemKind::Function => "function",
            ItemKind::Constant => "constant",
            ItemKind::Trait => "trait",
            ItemKind::AssocType => "assoc_type",
            ItemKind::AssocConst => "assoc_const",
            ItemKind::Macro => "macro",
            ItemKind::Impl => "impl",
        }
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ItemKind {
    type Error = nojson::JsonParseError;

    fn try_from(kind: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        match kind.to_unquoted_string_str()?.as_ref() {
            "module" => Ok(ItemKind::Module),
            "use" => Ok(ItemKind::Use),
            "enum" => Ok(ItemKind::Enum),
            "variant" => Ok(ItemKind::Variant),
            "struct" => Ok(ItemKind::Struct),
            "type_alias" => Ok(ItemKind::TypeAlias),
            "function" => Ok(ItemKind::Function),
            "constant" => Ok(ItemKind::Constant),
            "trait" => Ok(ItemKind::Trait),
            "assoc_type" => Ok(ItemKind::AssocType),
            "assoc_const" => Ok(ItemKind::AssocConst),
            "macro" => Ok(ItemKind::Macro),
            "impl" => Ok(ItemKind::Impl),
            _ => Err(kind.invalid(format!("unknown item kind"))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PublicItem {
    pub path: ItemPath,
    pub kind: ItemKind,
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
        let item = self.json.get_value_by_index(item_index.get()).expect("bug");

        let (kind, inner) = item
            .to_member("inner")?
            .required()?
            .to_object()?
            .next()
            .ok_or_else(|| item.invalid("empty inner"))?;
        let kind = ItemKind::try_from(kind)?;

        match kind {
            ItemKind::Module => self.visit_module(path, item_index, inner.try_into()?)?,
            _ => todo!(),
        }

        Ok(())
    }

    fn visit_module(
        &mut self,
        path: &mut ItemPath,
        _item_index: JsonValueIndex,
        _inner_index: JsonValueIndex,
    ) -> Result<(), nojson::JsonParseError> {
        // TODO: implement module visiting
        Ok(())
    }
}
