#[derive(Debug)]
pub struct ConstantView<'a> {
    doc: &'a crate::doc::CrateDoc,
    item: &'a crate::doc::Item,
}

impl<'a> ConstantView<'a> {
    pub fn new(doc: &'a crate::doc::CrateDoc, item: &'a crate::doc::Item) -> Self {
        assert_eq!(item.kind, crate::doc::ItemKind::Constant);
        Self { doc, item }
    }

    pub fn name(&self) -> &str {
        self.item.name.as_ref().expect("bug")
    }

    pub fn ty(&self) -> Result<String, nojson::JsonParseError> {
        let inner = self.item.inner(&self.doc.json);
        let ty = inner.to_member("type")?.required()?;
        format_type(ty, &self.doc)
    }
}

#[derive(Debug)]
pub struct AssocConstView<'a> {
    doc: &'a crate::doc::CrateDoc,
    item: &'a crate::doc::Item,
}

impl<'a> AssocConstView<'a> {
    pub fn new(doc: &'a crate::doc::CrateDoc, item: &'a crate::doc::Item) -> Self {
        assert_eq!(item.kind, crate::doc::ItemKind::AssocConst);
        Self { doc, item }
    }

    pub fn name(&self) -> &str {
        self.item.name.as_ref().expect("bug")
    }

    pub fn ty(&self) -> Result<String, nojson::JsonParseError> {
        let inner = self.item.inner(&self.doc.json);
        let ty = inner.to_member("type")?.required()?;
        format_type(ty, &self.doc)
    }
}

fn format_type(
    ty: nojson::RawJsonValue,
    doc: &crate::doc::CrateDoc,
) -> Result<String, nojson::JsonParseError> {
    if let Some(generic) = ty.to_member("generic")?.get() {
        // {"generic":"Self"}
        Ok(generic.try_into()?)
    } else if let Some(resolved) = ty.to_member("resolved_path")?.get() {
        // {"resolved_path":{"path":"FlagSpec","id":323,"args":null}}
        let path: String = resolved.to_member("path")?.required()?.try_into()?;
        Ok(path)
    } else if let Some(primitive) = ty.to_member("primitive")?.get() {
        // {"primitive":"bool"}
        Ok(primitive.try_into()?)
    } else {
        // Fallback: return the raw JSON representation
        Ok(ty.to_string())
    }
}
