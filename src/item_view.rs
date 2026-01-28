#[derive(Debug)]
pub struct TypeView<'a> {
    doc: &'a crate::doc::CrateDoc,
    item: &'a crate::doc::Item,
}

impl<'a> TypeView<'a> {
    pub fn new(doc: &'a crate::doc::CrateDoc, item: &'a crate::doc::Item) -> Self {
        Self { doc, item }
    }

    pub fn name(&self) -> Result<String, nojson::JsonParseError> {
        let name = self.item.name.as_ref().expect("bug");
        let inner = self.item.inner(&self.doc.json);

        // Check if there are generics to append
        if let Some(generics) = inner.to_member("generics")?.get() {
            let params = generics.to_member("params")?.required()?;
            let mut generic_names = Vec::new();

            for param in params.to_array()? {
                let param_name: String = param.to_member("name")?.required()?.try_into()?;
                generic_names.push(param_name);
            }

            if !generic_names.is_empty() {
                Ok(format!("{}<{}>", name, generic_names.join(", ")))
            } else {
                Ok(name.clone())
            }
        } else {
            Ok(name.clone())
        }
    }

    pub fn ty(&self) -> Result<String, nojson::JsonParseError> {
        let inner = self.item.inner(&self.doc.json);
        let ty = inner.to_member("type")?.required()?;
        format_type(ty, &self.doc)
    }
}

#[derive(Debug)]
pub struct ConstantView<'a> {
    doc: &'a crate::doc::CrateDoc,
    item: &'a crate::doc::Item,
}

impl<'a> ConstantView<'a> {
    pub fn new(doc: &'a crate::doc::CrateDoc, item: &'a crate::doc::Item) -> Self {
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

pub type AssocConstView<'a> = ConstantView<'a>;

fn format_type(
    ty: nojson::RawJsonValue,
    doc: &crate::doc::CrateDoc,
) -> Result<String, nojson::JsonParseError> {
    if let Some(generic) = ty.to_member("generic")?.get() {
        // {"generic":"Self"}
        Ok(generic.try_into()?)
    } else if let Some(resolved) = ty.to_member("resolved_path")?.get() {
        let path: String = resolved.to_member("path")?.required()?.try_into()?;

        // Check if this resolved_path has generic arguments
        if let Some(args) = resolved.to_member("args")?.get()
            && !args.kind().is_null()
        {
            // Handle generic arguments
            let mut formatted_args = Vec::new();

            if let Some(angle_bracketed) = args.to_member("angle_bracketed")?.get() {
                let args_array = angle_bracketed.to_member("args")?.required()?;

                for arg in args_array.to_array()? {
                    // Handle type arguments
                    if let Some(arg_type) = arg.to_member("type")?.get() {
                        formatted_args.push(format_type(arg_type, doc)?);
                    }
                    // Handle lifetime arguments
                    else if let Some(lifetime) = arg.to_member("lifetime")?.get() {
                        let lifetime_str: String = lifetime.try_into()?;
                        formatted_args.push(lifetime_str);
                    }
                }
            }

            if !formatted_args.is_empty() {
                Ok(format!("{}<{}>", path, formatted_args.join(", ")))
            } else {
                Ok(path)
            }
        } else {
            // {"resolved_path":{"path":"FlagSpec","id":323,"args":null}}
            Ok(path)
        }
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
