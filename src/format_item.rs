pub fn format_function_to_string(
    doc: &crate::doc::CrateDoc,
    item: &crate::doc::Item,
) -> crate::Result<String> {
    let mut buffer = Vec::new();
    let mut formatter = FunctionFormatter::new(&mut buffer, doc, item);
    formatter.format()?;
    Ok(String::from_utf8(buffer).expect("bug"))
}

#[derive(Debug)]
pub struct FunctionFormatter<'a, W> {
    writer: W,
    doc: &'a crate::doc::CrateDoc,
    item: &'a crate::doc::Item,
}

impl<'a, W: std::io::Write> FunctionFormatter<'a, W> {
    pub fn new(writer: W, doc: &'a crate::doc::CrateDoc, item: &'a crate::doc::Item) -> Self {
        Self { writer, doc, item }
    }

    pub fn format(&mut self) -> crate::Result<()> {
        let inner = self
            .item
            .inner(&self.doc.json)
            .to_member("function")?
            .required()?;
        self.format_function_signature(inner)?;
        Ok(())
    }

    fn format_function_signature(&mut self, function: nojson::RawJsonValue) -> crate::Result<()> {
        let sig = function.to_member("sig")?.required()?;

        // Format header (const, unsafe, async)
        self.format_function_header(sig)?;

        // Format function name
        let name = self.item.name.as_ref().expect("bug");
        write!(self.writer, "fn {}", name)?;

        // Format generics
        let generics = function.to_member("generics")?;
        if let Some(g) = generics.get() {
            self.format_generics(g)?;
        }

        // Format parameters
        write!(self.writer, "(")?;
        self.format_function_inputs(sig)?;
        write!(self.writer, ")")?;

        // Format return type
        self.format_function_output(sig)?;

        // Format where clauses
        if let Some(g) = generics.get() {
            self.format_where_clauses(g)?;
        }

        Ok(())
    }

    fn format_function_header(&mut self, sig: nojson::RawJsonValue) -> crate::Result<()> {
        let header = sig.to_member("header")?.required()?;

        let is_const: bool = header.to_member("is_const")?.required()?.try_into()?;
        let is_unsafe: bool = header.to_member("is_unsafe")?.required()?.try_into()?;
        let is_async: bool = header.to_member("is_async")?.required()?.try_into()?;

        if is_const {
            write!(self.writer, "const ")?;
        }
        if is_unsafe {
            write!(self.writer, "unsafe ")?;
        }
        if is_async {
            write!(self.writer, "async ")?;
        }

        Ok(())
    }

    fn format_generics(&mut self, generics: nojson::RawJsonValue) -> crate::Result<()> {
        let params = generics.to_member("params")?;

        if let Some(params_array) = params.get() {
            let params_list: Vec<_> = params_array.to_array()?.collect();

            if !params_list.is_empty() {
                write!(self.writer, "<")?;

                for (i, param) in params_list.iter().enumerate() {
                    if i > 0 {
                        write!(self.writer, ", ")?;
                    }

                    let param_name = param
                        .to_member("name")?
                        .required()?
                        .to_unquoted_string_str()?;
                    write!(self.writer, "{}", param_name)?;

                    // Format bounds if present
                    let kind = param.to_member("kind")?;
                    if let Some(kind_obj) = kind.get() {
                        if let Some(type_bounds) = kind_obj.to_member("type")?.get() {
                            self.format_type_bounds(type_bounds)?;
                        }
                    }
                }

                write!(self.writer, ">")?;
            }
        }

        Ok(())
    }

    fn format_type_bounds(&mut self, type_obj: nojson::RawJsonValue) -> crate::Result<()> {
        let bounds = type_obj.to_member("bounds")?;

        if let Some(bounds_array) = bounds.get() {
            let bounds_list: Vec<_> = bounds_array.to_array()?.collect();

            if !bounds_list.is_empty() {
                write!(self.writer, ": ")?;

                for (i, bound) in bounds_list.iter().enumerate() {
                    if i > 0 {
                        write!(self.writer, " + ")?;
                    }

                    if let Some(trait_bound) = bound.to_member("trait_bound")?.get() {
                        let trait_info = trait_bound.to_member("trait")?.required()?;
                        let trait_path = trait_info
                            .to_member("path")?
                            .required()?
                            .to_unquoted_string_str()?;
                        write!(self.writer, "{}", trait_path)?;

                        // Format trait generic args if present
                        if let Some(args) = trait_info.to_member("args")?.get() {
                            if !args.kind().is_null() {
                                self.format_trait_args(args)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn format_trait_args(&mut self, args: nojson::RawJsonValue) -> crate::Result<()> {
        if let Some(angle_bracketed) = args.to_member("angle_bracketed")?.get() {
            write!(self.writer, "<")?;

            let args_array = angle_bracketed.to_member("args")?;
            if let Some(args_list) = args_array.get() {
                for (i, arg) in args_list.to_array()?.enumerate() {
                    if i > 0 {
                        write!(self.writer, ", ")?;
                    }

                    if let Some(arg_type) = arg.to_member("type")?.get() {
                        let formatted = crate::format_type::format_to_string(self.doc, arg_type)?;
                        write!(self.writer, "{}", formatted)?;
                    }
                }
            }

            write!(self.writer, ">")?;
        }

        Ok(())
    }

    fn format_function_inputs(&mut self, sig: nojson::RawJsonValue) -> crate::Result<()> {
        let inputs = sig.to_member("inputs")?.required()?;

        for (i, input_pair) in inputs.to_array()?.enumerate() {
            if i > 0 {
                write!(self.writer, ", ")?;
            }

            let input_array = input_pair.to_array()?;
            let input_items: Vec<_> = input_array.collect();

            if input_items.len() >= 2 {
                let param_name = input_items[0].to_unquoted_string_str()?;
                let param_type = input_items[1];

                write!(self.writer, "{}: ", param_name)?;
                let formatted_type = crate::format_type::format_to_string(self.doc, param_type)?;
                write!(self.writer, "{}", formatted_type)?;
            }
        }

        Ok(())
    }

    fn format_function_output(&mut self, sig: nojson::RawJsonValue) -> crate::Result<()> {
        let output = sig.to_member("output")?;

        if let Some(output_type) = output.get() {
            if !output_type.kind().is_null() {
                write!(self.writer, " -> ")?;
                let formatted_type = crate::format_type::format_to_string(self.doc, output_type)?;
                write!(self.writer, "{}", formatted_type)?;
            }
        }

        Ok(())
    }

    fn format_where_clauses(&mut self, generics: nojson::RawJsonValue) -> crate::Result<()> {
        let where_predicates = generics.to_member("where_predicates")?;

        if let Some(predicates) = where_predicates.get() {
            let predicates_list: Vec<_> = predicates.to_array()?.collect();

            if !predicates_list.is_empty() {
                write!(self.writer, "\nwhere\n    ")?;

                for (i, predicate) in predicates_list.iter().enumerate() {
                    if i > 0 {
                        write!(self.writer, ",\n    ")?;
                    }

                    // Format the where predicate
                    self.format_where_predicate(*predicate)?;
                }
            }
        }

        Ok(())
    }

    fn format_where_predicate(&mut self, predicate: nojson::RawJsonValue) -> crate::Result<()> {
        // Try to extract the left-hand side (type being constrained)
        if let Some(lhs) = predicate.to_member("lhs")?.get() {
            let formatted_lhs = crate::format_type::format_to_string(self.doc, lhs)?;
            write!(self.writer, "{}", formatted_lhs)?;
        }

        // Format the bounds
        let bounds = predicate.to_member("bounds")?;
        if let Some(bounds_array) = bounds.get() {
            let bounds_list: Vec<_> = bounds_array.to_array()?.collect();

            for (i, bound) in bounds_list.iter().enumerate() {
                if i == 0 {
                    write!(self.writer, ": ")?;
                } else {
                    write!(self.writer, " + ")?;
                }

                if let Some(trait_bound) = bound.to_member("trait_bound")?.get() {
                    let trait_info = trait_bound.to_member("trait")?.required()?;
                    let trait_path = trait_info
                        .to_member("path")?
                        .required()?
                        .to_unquoted_string_str()?;
                    write!(self.writer, "{}", trait_path)?;

                    if let Some(args) = trait_info.to_member("args")?.get() {
                        if !args.kind().is_null() {
                            self.format_trait_args(args)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // todo: test
    //  {"sig":{"inputs":[["key",{"generic":"K"}]],"output":{"resolved_path":{"path":"Result","id":406,"args":{"angle_bracketed":{"args":[{"type":{"resolved_path":{"path":"String","id":447,"args":null}}},{"type":{"resolved_path":{"path":"std::env::VarError","id":628,"args":null}}}],"constraints":[]}}}},"is_c_variadic":false},"generics":{"params":[{"name":"K","kind":{"type":{"bounds":[{"trait_bound":{"trait":{"path":"AsRef","id":629,"args":{"angle_bracketed":{"args":[{"type":{"resolved_path":{"path":"std::ffi::OsStr","id":630,"args":null}}}],"constraints":[]}}},"generic_params":[],"modifier":"none"}},{"trait_bound":{"trait":{"path":"AsRef","id":629,"args":{"angle_bracketed":{"args":[{"type":{"primitive":"str"}}],"constraints":[]}}},"generic_params":[],"modifier":"none"}}],"default":null,"is_synthetic":false}}}],"where_predicates":[]},"header":{"is_const":false,"is_unsafe":false,"is_async":false,"abi":"Rust"},"has_body":true}
}
