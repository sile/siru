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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Module,
    Use,
    Enum,
    Variant,
    Struct,
    StructField,
    TypeAlias,
    Function,
    Constant,
    Trait,
    AssocType,
    AssocConst,
    Macro,
    Impl,
}

impl std::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ItemKind {
    pub const KEYWORDS: &'static str = "mod|enum|variant|struct|field|type|fn|const|trait|macro";

    pub fn parse_keyword_str(s: &str) -> Option<Vec<Self>> {
        match s {
            "mod" => Some(vec![ItemKind::Module]),
            "enum" => Some(vec![ItemKind::Enum]),
            "variant" => Some(vec![ItemKind::Variant]),
            "struct" => Some(vec![ItemKind::Struct]),
            "field" => Some(vec![ItemKind::StructField]),
            "type" => Some(vec![ItemKind::TypeAlias, ItemKind::AssocType]),
            "fn" => Some(vec![ItemKind::Function]),
            "const" => Some(vec![ItemKind::Constant, ItemKind::AssocConst]),
            "trait" => Some(vec![ItemKind::Trait]),
            "macro" => Some(vec![ItemKind::Macro]),
            // NOTE: Filters out unnamed items
            // "use" => Some(vec![ItemKind::Use]),
            // "impl" => Some(vec![ItemKind::Impl]),
            _ => None,
        }
    }

    /// Returns the Rust keyword representation for this item kind (as it appears in source code)
    pub fn as_keyword_str(self) -> &'static str {
        match self {
            ItemKind::Module => "mod",
            ItemKind::Use => "use",
            ItemKind::Enum => "enum",
            ItemKind::Variant => "variant",
            ItemKind::Struct => "struct",
            ItemKind::StructField => "field",
            ItemKind::TypeAlias => "type",
            ItemKind::Function => "fn",
            ItemKind::Constant => "const",
            ItemKind::Trait => "trait",
            ItemKind::AssocType => "type",
            ItemKind::AssocConst => "const",
            ItemKind::Macro => "macro",
            ItemKind::Impl => "impl",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ItemKind::Module => "module",
            ItemKind::Use => "use",
            ItemKind::Enum => "enum",
            ItemKind::Variant => "variant",
            ItemKind::Struct => "struct",
            ItemKind::StructField => "struct_field",
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
            "struct_field" => Ok(ItemKind::StructField),
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
pub struct Item {
    pub name: Option<String>,
    pub kind: ItemKind,
    pub is_public: bool,
    pub docs_index: Option<JsonValueIndex>,
    pub deprecation_index: Option<JsonValueIndex>,
    pub inner_index: JsonValueIndex,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Item {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let name = value.to_member("name")?.required()?.try_into()?;
        let (kind, inner) = value
            .to_member("inner")?
            .required()?
            .to_object()?
            .next()
            .ok_or_else(|| value.invalid("empty inner"))?;
        let kind = kind.try_into()?;
        let inner_index = inner.try_into()?;
        let visibility = value
            .to_member("visibility")?
            .required()?
            .to_unquoted_string_str()?;
        let docs_index = value.to_member("docs")?.required()?.try_into()?;
        let deprecation_index = value.to_member("deprecation")?.required()?.try_into()?;

        let is_public =
            visibility.as_ref() == "public" || matches!(kind, ItemKind::Impl | ItemKind::Variant);
        Ok(Self {
            name,
            kind,
            is_public,
            docs_index,
            deprecation_index,
            inner_index,
        })
    }
}

impl Item {
    pub fn inner<'a>(&self, json: &'a nojson::RawJsonOwned) -> nojson::RawJsonValue<'a, 'a> {
        json.get_value_by_index(self.inner_index.get())
            .expect("bug")
    }

    pub fn docs(
        &self,
        json: &nojson::RawJsonOwned,
    ) -> Result<Option<String>, nojson::JsonParseError> {
        let Some(index) = self.docs_index else {
            return Ok(None);
        };
        let value = json.get_value_by_index(index.get()).expect("bug");
        Ok(Some(value.try_into()?))
    }

    pub fn deprecation_note(
        &self,
        json: &nojson::RawJsonOwned,
    ) -> Result<Option<String>, nojson::JsonParseError> {
        let Some(index) = self.deprecation_index else {
            return Ok(None);
        };
        let value = json.get_value_by_index(index.get()).expect("bug");
        let note: Option<String> = value.to_member("note")?.try_into()?;
        Ok(Some(note.unwrap_or_default()))
    }
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

impl std::fmt::Display for ItemPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, segment) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, "::")?;
            }
            write!(f, "{}", segment)?;
        }
        Ok(())
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
    pub show_items: Vec<(ItemPath, Item)>,
    pub public_item_count: usize,
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
            show_items: Vec::new(),
            public_item_count: 0,
        };
        let mut collector = PublicItemCollector::new(&this.json, &this.items);
        let root_module_value = this
            .json
            .get_value_by_index(this.root_module_index.get())
            .expect("bug");
        collector.collect(root_module_value)?;
        this.show_items = collector.public_items;
        this.public_item_count = this.show_items.len();
        Ok(this)
    }
}

struct PublicItemCollector<'a> {
    json: &'a nojson::RawJsonOwned,
    items: &'a CrateItems,
    public_items: Vec<(ItemPath, Item)>,
}

impl<'a> PublicItemCollector<'a> {
    fn new(json: &'a nojson::RawJsonOwned, items: &'a CrateItems) -> Self {
        Self {
            json,
            items,
            public_items: Vec::new(),
        }
    }

    fn collect(
        &mut self,
        root_module_value: nojson::RawJsonValue<'a, 'a>,
    ) -> Result<(), nojson::JsonParseError> {
        let mut path = ItemPath(Vec::new());
        self.visit_item(&mut path, root_module_value, false)?;
        Ok(())
    }

    fn visit_item(
        &mut self,
        path: &mut ItemPath,
        item_value: nojson::RawJsonValue<'a, 'a>,
        force_public: bool,
    ) -> Result<(), nojson::JsonParseError> {
        let item = Item::try_from(item_value)?;

        if !item.is_public && !force_public {
            return Ok(());
        }

        if let Some(name) = &item.name {
            path.0.push(name.clone());
            self.public_items.push((path.clone(), item.clone()));
        }

        let inner = item.inner(self.json);
        match item.kind {
            ItemKind::Module => {
                for item_id_value in inner.to_member("items")?.required()?.to_array()? {
                    let item_value = self.items.get(self.json, item_id_value)?;
                    self.visit_item(path, item_value, false)?;
                }
            }
            ItemKind::Enum => {
                for item_id_value in inner.to_member("variants")?.required()?.to_array()? {
                    let item_value = self.items.get(self.json, item_id_value)?;
                    self.visit_item(path, item_value, false)?;
                }
                for item_id_value in inner.to_member("impls")?.required()?.to_array()? {
                    let item_value = self.items.get(self.json, item_id_value)?;
                    self.visit_item(path, item_value, false)?;
                }
            }
            ItemKind::Struct => {
                let kind_value = inner.to_member("kind")?.required()?;
                if let Some((kind, plain)) = kind_value.to_object().ok().and_then(|mut o| o.next())
                    && kind.to_unquoted_string_str()? == "plain"
                {
                    for field_id_value in plain.to_member("fields")?.required()?.to_array()? {
                        let field_value = self.items.get(self.json, field_id_value)?;
                        self.visit_item(path, field_value, false)?;
                    }
                }

                for item_id_value in inner.to_member("impls")?.required()?.to_array()? {
                    let item_value = self.items.get(self.json, item_id_value)?;
                    self.visit_item(path, item_value, false)?;
                }
            }
            ItemKind::Trait => {
                for item_id_value in inner.to_member("items")?.required()?.to_array()? {
                    let item_value = self.items.get(self.json, item_id_value)?;
                    self.visit_item(path, item_value, true)?;
                }
            }
            ItemKind::Impl => {
                for item_id_value in inner.to_member("items")?.required()?.to_array()? {
                    let item_value = self.items.get(self.json, item_id_value)?;
                    self.visit_item(path, item_value, false)?;
                }
            }
            ItemKind::Use => {
                let is_glob: bool = inner.to_member("is_glob")?.required()?.try_into()?;
                if !is_glob {
                    let target_id_value = inner.to_member("id")?.required()?;
                    let target_item_value = self.items.get(self.json, target_id_value)?;
                    self.visit_item(path, target_item_value, false)?;
                }
            }
            // Leaf items with no children to visit
            ItemKind::Variant
            | ItemKind::StructField
            | ItemKind::TypeAlias
            | ItemKind::Function
            | ItemKind::Constant
            | ItemKind::AssocType
            | ItemKind::AssocConst
            | ItemKind::Macro => {}
        }

        if item.name.is_some() {
            path.0.pop();
        }

        Ok(())
    }
}
