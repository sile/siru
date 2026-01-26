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
