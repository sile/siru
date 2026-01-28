pub fn format_signature(
    doc: &crate::doc::CrateDoc,
    item: &crate::doc::Item,
) -> crate::Result<String> {
    let mut formatter = ItemFormatter::new(doc, item);
    formatter.format()
}

#[derive(Debug)]
pub struct ItemFormatter<'a> {
    doc: &'a crate::doc::CrateDoc,
    item: &'a crate::doc::Item,
}

impl<'a> ItemFormatter<'a> {
    pub fn new(doc: &'a crate::doc::CrateDoc, item: &'a crate::doc::Item) -> Self {
        Self { doc, item }
    }

    pub fn format(&mut self) -> crate::Result<String> {
        match self.item.kind {
            crate::doc::ItemKind::Function => self.format_function(),
            crate::doc::ItemKind::TypeAlias | crate::doc::ItemKind::AssocType => {
                self.format_type_alias()
            }
            crate::doc::ItemKind::Primitive => self.format_primitive(),
            crate::doc::ItemKind::Constant | crate::doc::ItemKind::AssocConst => {
                self.format_constant()
            }
            crate::doc::ItemKind::Module => self.format_module(),
            crate::doc::ItemKind::Macro => self.format_macro(),
            crate::doc::ItemKind::ProcMacro => self.format_proc_macro(),
            crate::doc::ItemKind::StructField => self.format_struct_field(),
            kind => Err(format!("Unsupported item kind: {:?}", kind).into()),
        }
    }

    fn format_function(&mut self) -> crate::Result<String> {
        let name = self.item.name.as_ref().expect("bug");
        let inner = self.item.inner(&self.doc.json);
        let sig = inner.to_member("sig")?.required()?;

        let mut output = String::from("fn ");
        output.push_str(name);

        // Format generic parameters if present
        self.append_generics(&mut output, inner)?;

        // Format inputs
        self.append_inputs(&mut output, sig)?;

        // Format output
        output.push_str(" -> ");
        let output_ty = sig.to_member("output")?.required()?;
        output.push_str(&crate::format_type::format_to_string(self.doc, output_ty)?);

        Ok(output)
    }

    fn format_type_alias(&mut self) -> crate::Result<String> {
        let view = crate::item_view::TypeView::new(self.doc, self.item);
        if let Some(ty) = view.ty()? {
            Ok(format!("type {} = {};", view.name()?, ty))
        } else {
            Ok(format!("type {};", view.name()?))
        }
    }

    fn format_primitive(&mut self) -> crate::Result<String> {
        let view = crate::item_view::PrimitiveView::new(self.doc, self.item);
        Ok(format!("type {};", view.name()))
    }

    fn format_constant(&mut self) -> crate::Result<String> {
        let view = crate::item_view::ConstantView::new(self.doc, self.item);
        Ok(format!("const {}: {};", view.name(), view.ty()?))
    }

    fn format_module(&mut self) -> crate::Result<String> {
        let view = crate::item_view::ModuleView::new(self.doc, self.item);
        let child_count = view.child_count()?;
        Ok(format!(
            "mod {} {{ /* {} items */ }}",
            view.name(),
            child_count
        ))
    }

    fn format_macro(&mut self) -> crate::Result<String> {
        let inner = self.item.inner(&self.doc.json);
        Ok(inner.to_unquoted_string_str()?.to_string())
    }

    fn format_proc_macro(&mut self) -> crate::Result<String> {
        let view = crate::item_view::ProcMacroView::new(self.doc, self.item);
        view.derive_attribute()
    }

    fn format_struct_field(&mut self) -> crate::Result<String> {
        let view = crate::item_view::FieldView::new(self.doc, self.item);
        Ok(format!("  {}: {}", view.name(), view.ty()?))
    }

    fn append_generics(
        &mut self,
        output: &mut String,
        inner: nojson::RawJsonValue,
    ) -> crate::Result<()> {
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
        Ok(())
    }

    fn append_inputs(
        &mut self,
        output: &mut String,
        sig: nojson::RawJsonValue,
    ) -> crate::Result<()> {
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
                        let ty_str = crate::format_type::format_to_string(self.doc, ty_val)?;
                        output.push_str(&ty_str);
                    }
                    first = false;
                }
            }
        }
        output.push(')');
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // todo: test
    //  {"sig":{"inputs":[["key",{"generic":"K"}]],"output":{"resolved_path":{"path":"Result","id":406,"args":{"angle_bracketed":{"args":[{"type":{"resolved_path":{"path":"String","id":447,"args":null}}},{"type":{"resolved_path":{"path":"std::env::VarError","id":628,"args":null}}}],"constraints":[]}}}},"is_c_variadic":false},"generics":{"params":[{"name":"K","kind":{"type":{"bounds":[{"trait_bound":{"trait":{"path":"AsRef","id":629,"args":{"angle_bracketed":{"args":[{"type":{"resolved_path":{"path":"std::ffi::OsStr","id":630,"args":null}}}],"constraints":[]}}},"generic_params":[],"modifier":"none"}},{"trait_bound":{"trait":{"path":"AsRef","id":629,"args":{"angle_bracketed":{"args":[{"type":{"primitive":"str"}}],"constraints":[]}}},"generic_params":[],"modifier":"none"}}],"default":null,"is_synthetic":false}}}],"where_predicates":[]},"header":{"is_const":false,"is_unsafe":false,"is_async":false,"abi":"Rust"},"has_body":true}
}
