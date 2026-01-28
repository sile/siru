// todo: add TypeView
// example: {"type":{"resolved_path":{"path":"std::result::Result","id":47,"args":{"angle_bracketed":{"args":[{"type":{"generic":"T"}},{"type":{"resolved_path":{"path":"Error","id":107,"args":null}}}],"constraints":[]}}}},"generics":{"params":[{"name":"T","kind":{"type":{"bounds":[],"default":null,"is_synthetic":false}}}],"where_predicates":[]}}

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
    } else if let Some(resolved) = ty.to_member("resolved_path")?.get()
        && resolved.to_member("args")?.required()?.kind().is_null()
    {
        // {"resolved_path":{"path":"FlagSpec","id":323,"args":null}}
        let path: String = resolved.to_member("path")?.required()?.try_into()?;
        Ok(path)
    } else if let Some(primitive) = ty.to_member("primitive")?.get() {
        // {"primitive":"bool"}
        Ok(primitive.try_into()?)
    } else if let Some(borrowed_ref) = ty.to_member("borrowed_ref")?.get()
        && borrowed_ref
            .to_member("lifetime")?
            .required()?
            .kind()
            .is_null()
    {
        // {"borrowed_ref":{"lifetime":null,"is_mutable":false,"type":{"primitive":"str"}}}
        let is_mutable: bool = borrowed_ref
            .to_member("is_mutable")?
            .required()?
            .try_into()?;
        let inner_type = borrowed_ref.to_member("type")?.required()?;
        let inner_formatted = format_type(inner_type, doc)?;
        let prefix = if is_mutable { "&mut " } else { "&" };
        Ok(format!("{}{}", prefix, inner_formatted))
    } else if let Some(tuple) = ty.to_member("tuple")?.get() {
        // {"tuple":[{"primitive":"u8"},{"primitive":"u8"},{"primitive":"u8"}]}
        let mut formatted_types = Vec::new();
        for element_type in tuple.to_array()? {
            formatted_types.push(format_type(element_type, doc)?);
        }
        Ok(format!("({})", formatted_types.join(", ")))
    } else {
        // Fallback: return the raw JSON representation
        Ok(ty.to_string())
    }
}
