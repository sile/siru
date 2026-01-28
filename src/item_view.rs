#[derive(Debug)]
pub struct FunctionView<'a> {
    doc: &'a crate::doc::CrateDoc,
    item: &'a crate::doc::Item,
}

impl<'a> FunctionView<'a> {
    pub fn new(doc: &'a crate::doc::CrateDoc, item: &'a crate::doc::Item) -> Self {
        Self { doc, item }
    }

    pub fn name(&self) -> &str {
        self.item.name.as_ref().expect("bug")
    }

    pub fn signature(&self) -> crate::Result<String> {
        let inner = self.item.inner(&self.doc.json);
        let sig = inner.to_member("sig")?.required()?;
        // todo: add another module like format_tyoe.rs to format function signature
        let mut output = String::from("fn ");
        output.push_str(self.name());

        // Format generic parameters if present
        if let Some(generics) = inner.to_member("generics")?.get() {
            if let Some(params) = generics.to_member("params")?.get() {
                if !params.kind().is_null() {
                    output.push('<');
                    let mut first = true;
                    for param in params.to_array()? {
                        if !first {
                            output.push_str(", ");
                        }
                        let param_name: String = param.to_member("name")?.required()?.try_into()?;
                        output.push_str(&param_name);
                        first = false;
                    }
                    output.push('>');
                }
            }
        }

        // Format inputs
        output.push('(');
        if let Some(inputs) = sig.to_member("inputs")?.get() {
            if !inputs.kind().is_null() {
                let mut first = true;
                for input in inputs.to_array()? {
                    if !first {
                        output.push_str(", ");
                    }
                    if let Some(name_val) = input.to_array()?.next() {
                        let param_name: String = name_val.try_into()?;
                        output.push_str(&param_name);
                        output.push_str(": ");
                    }
                    if let Some(ty_val) = input.to_array()?.nth(1) {
                        let ty_str = crate::format_type::format_to_string(&self.doc, ty_val)?;
                        output.push_str(&ty_str);
                    }
                    first = false;
                }
            }
        }
        output.push(')');

        // Format output
        output.push_str(" -> ");
        let output_ty = sig.to_member("output")?.required()?;
        output.push_str(&crate::format_type::format_to_string(&self.doc, output_ty)?);

        Ok(output)
    }
}

#[derive(Debug)]
pub struct FieldView<'a> {
    doc: &'a crate::doc::CrateDoc,
    item: &'a crate::doc::Item,
}

impl<'a> FieldView<'a> {
    pub fn new(doc: &'a crate::doc::CrateDoc, item: &'a crate::doc::Item) -> Self {
        Self { doc, item }
    }

    pub fn name(&self) -> &str {
        self.item.name.as_ref().expect("bug")
    }

    pub fn ty(&self) -> crate::Result<String> {
        let inner = self.item.inner(&self.doc.json);
        crate::format_type::format_to_string(&self.doc, inner)
    }
}

#[derive(Debug)]
pub struct ModuleView<'a> {
    doc: &'a crate::doc::CrateDoc,
    item: &'a crate::doc::Item,
}

impl<'a> ModuleView<'a> {
    pub fn new(doc: &'a crate::doc::CrateDoc, item: &'a crate::doc::Item) -> Self {
        Self { doc, item }
    }

    pub fn name(&self) -> &str {
        self.item.name.as_ref().expect("bug")
    }

    pub fn child_count(&self) -> crate::Result<usize> {
        let inner = self.item.inner(&self.doc.json);
        let items = inner.to_member("items")?.required()?;
        Ok(items.to_array()?.count())
    }
}

#[derive(Debug)]
pub struct ProcMacroView<'a> {
    doc: &'a crate::doc::CrateDoc,
    item: &'a crate::doc::Item,
}

impl<'a> ProcMacroView<'a> {
    pub fn new(doc: &'a crate::doc::CrateDoc, item: &'a crate::doc::Item) -> Self {
        Self { doc, item }
    }

    pub fn name(&self) -> &str {
        self.item.name.as_ref().expect("bug")
    }

    pub fn derive_attribute(&self) -> crate::Result<String> {
        let inner = self.item.inner(&self.doc.json);
        let kind = inner
            .to_member("kind")?
            .required()?
            .to_unquoted_string_str()?;
        if kind == "derive" {
            Ok(format!("#[derive({})]", self.name()))
        } else {
            Ok(format!("{inner}"))
        }
    }
}

#[derive(Debug)]
pub struct PrimitiveView<'a> {
    doc: &'a crate::doc::CrateDoc,
    item: &'a crate::doc::Item,
}

impl<'a> PrimitiveView<'a> {
    pub fn new(doc: &'a crate::doc::CrateDoc, item: &'a crate::doc::Item) -> Self {
        Self { doc, item }
    }

    pub fn name(&self) -> &str {
        self.item.name.as_ref().expect("bug")
    }

    pub fn impls(&self) -> crate::Result<Vec<u64>> {
        let inner = self.item.inner(&self.doc.json);
        let impls = inner.to_member("impls")?.required()?;

        let mut impl_ids = Vec::new();
        for impl_id in impls.to_array()? {
            let id: u64 = impl_id.try_into()?;
            impl_ids.push(id);
        }

        Ok(impl_ids)
    }
}

#[derive(Debug)]
pub struct TypeView<'a> {
    doc: &'a crate::doc::CrateDoc,
    item: &'a crate::doc::Item,
}

impl<'a> TypeView<'a> {
    pub fn new(doc: &'a crate::doc::CrateDoc, item: &'a crate::doc::Item) -> Self {
        Self { doc, item }
    }

    pub fn name(&self) -> crate::Result<String> {
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

    pub fn ty(&self) -> crate::Result<Option<String>> {
        let inner = self.item.inner(&self.doc.json);
        let ty = inner.to_member("type")?.required()?;
        if ty.kind().is_null() {
            return Ok(None);
        }

        crate::format_type::format_to_string(&self.doc, ty).map(Some)
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

    pub fn ty(&self) -> crate::Result<String> {
        let inner = self.item.inner(&self.doc.json);
        let ty = inner.to_member("type")?.required()?;
        crate::format_type::format_to_string(&self.doc, ty)
    }
}
