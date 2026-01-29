pub fn format_to_string(
    doc: &crate::doc::CrateDoc,
    ty: nojson::RawJsonValue,
) -> crate::Result<String> {
    let mut buffer = Vec::new();
    let mut formatter = TypeFormatter::new(&mut buffer, doc);
    formatter.format(ty)?;
    Ok(String::from_utf8(buffer).expect("bug"))
}

#[derive(Debug)]
pub struct TypeFormatter<'a, W> {
    writer: W,
    #[expect(dead_code)]
    doc: &'a crate::doc::CrateDoc,
}

impl<'a, W: std::io::Write> TypeFormatter<'a, W> {
    pub fn new(writer: W, doc: &'a crate::doc::CrateDoc) -> Self {
        Self { writer, doc }
    }

    pub fn format(&mut self, ty: nojson::RawJsonValue) -> crate::Result<()> {
        self.format_type(ty)?;
        Ok(())
    }

    fn format_type(&mut self, ty: nojson::RawJsonValue) -> crate::Result<()> {
        if let Some(generic) = ty.to_member("generic")?.get() {
            self.format_generic(generic)
        } else if let Some(resolved) = ty.to_member("resolved_path")?.get() {
            self.format_resolved_path(resolved)
        } else if let Some(primitive) = ty.to_member("primitive")?.get() {
            self.format_primitive(primitive)
        } else if let Some(borrowed_ref) = ty.to_member("borrowed_ref")?.get() {
            self.format_borrowed_ref(borrowed_ref)
        } else if let Some(raw_pointer) = ty.to_member("raw_pointer")?.get() {
            self.format_raw_pointer(raw_pointer)
        } else if let Some(qualified_path) = ty.to_member("qualified_path")?.get() {
            self.format_qualified_path(qualified_path)
        } else if let Some(tuple) = ty.to_member("tuple")?.get() {
            self.format_tuple(tuple)
        } else if let Some(dyn_trait) = ty.to_member("dyn_trait")?.get() {
            self.format_dyn_trait(dyn_trait)
        } else if let Some(impl_trait) = ty.to_member("impl_trait")?.get() {
            self.format_impl_trait(impl_trait)
        } else if let Some(slice) = ty.to_member("slice")?.get() {
            self.format_slice(slice)
        } else if let Some(array) = ty.to_member("array")?.get() {
            self.format_array(array)
        } else if let Some(function_pointer) = ty.to_member("function_pointer")?.get() {
            self.format_function_pointer(function_pointer)
        } else {
            write!(self.writer, "{}", ty)?;
            Ok(())
        }
    }

    fn format_generic(&mut self, generic: nojson::RawJsonValue) -> crate::Result<()> {
        write!(self.writer, "{}", generic.to_unquoted_string_str()?)?;
        Ok(())
    }

    fn format_resolved_path(&mut self, resolved: nojson::RawJsonValue) -> crate::Result<()> {
        let path = resolved
            .to_member("path")?
            .required()?
            .to_unquoted_string_str()?;

        if let Some(args) = resolved.to_member("args")?.get()
            && !args.kind().is_null()
        {
            self.format_resolved_path_with_args(path, args)
        } else {
            write!(self.writer, "{}", path)?;
            Ok(())
        }
    }

    fn format_resolved_path_with_args(
        &mut self,
        path: std::borrow::Cow<str>,
        args: nojson::RawJsonValue,
    ) -> crate::Result<()> {
        write!(self.writer, "{}", path)?;

        let Some(angle_bracketed) = args.to_member("angle_bracketed")?.get() else {
            return Ok(());
        };

        self.format_angle_bracketed_args(angle_bracketed)
    }

    fn format_angle_bracketed_args(&mut self, args: nojson::RawJsonValue) -> crate::Result<()> {
        write!(self.writer, "<")?;
        let args_array = args.to_member("args")?.required()?;
        for (i, arg) in args_array.to_array()?.enumerate() {
            if i > 0 {
                write!(self.writer, ", ")?;
            }

            if let Some(arg_type) = arg.to_member("type")?.get() {
                self.format_type(arg_type)?;
            } else if let Some(lifetime) = arg.to_member("lifetime")?.get() {
                let lifetime_str = lifetime.to_unquoted_string_str()?;
                write!(self.writer, "{}", lifetime_str)?;
            }
        }
        write!(self.writer, ">")?;

        Ok(())
    }

    fn format_primitive(&mut self, primitive: nojson::RawJsonValue) -> crate::Result<()> {
        let formatted = primitive.to_unquoted_string_str()?;
        write!(self.writer, "{}", formatted)?;
        Ok(())
    }

    fn format_borrowed_ref(&mut self, borrowed_ref: nojson::RawJsonValue) -> crate::Result<()> {
        let is_mutable: bool = borrowed_ref
            .to_member("is_mutable")?
            .required()?
            .try_into()?;
        let lifetime = borrowed_ref.to_member("lifetime")?;
        let inner_type = borrowed_ref.to_member("type")?.required()?;

        let prefix = if is_mutable { "&mut " } else { "&" };
        write!(self.writer, "{}", prefix)?;

        // Write lifetime if present
        if let Some(lifetime_val) = lifetime.get()
            && !lifetime_val.kind().is_null() {
                let lifetime_str = lifetime_val.to_unquoted_string_str()?;
                write!(self.writer, "{} ", lifetime_str)?;
            }

        self.format_type(inner_type)
    }

    fn format_raw_pointer(&mut self, raw_pointer: nojson::RawJsonValue) -> crate::Result<()> {
        self.format_prefixed_type(raw_pointer, "*mut ", "*const ")
    }

    fn format_prefixed_type(
        &mut self,
        obj: nojson::RawJsonValue,
        mutable_prefix: &str,
        const_prefix: &str,
    ) -> crate::Result<()> {
        let is_mutable: bool = obj.to_member("is_mutable")?.required()?.try_into()?;
        let inner_type = obj.to_member("type")?.required()?;
        let prefix = if is_mutable {
            mutable_prefix
        } else {
            const_prefix
        };
        write!(self.writer, "{}", prefix)?;
        self.format_type(inner_type)
    }

    fn format_qualified_path(&mut self, qualified_path: nojson::RawJsonValue) -> crate::Result<()> {
        let name = qualified_path
            .to_member("name")?
            .required()?
            .to_unquoted_string_str()?;
        let self_type = qualified_path.to_member("self_type")?.required()?;
        let trait_info = qualified_path.to_member("trait")?.required()?;
        let trait_path = trait_info
            .to_member("path")?
            .required()?
            .to_unquoted_string_str()?;

        write!(self.writer, "<")?;
        self.format_type(self_type)?;
        write!(self.writer, " as {}>::{}", trait_path, name)?;

        Ok(())
    }

    fn format_tuple(&mut self, tuple: nojson::RawJsonValue) -> crate::Result<()> {
        write!(self.writer, "(")?;
        let mut first = true;
        for element_type in tuple.to_array()? {
            if !first {
                write!(self.writer, ", ")?;
            }
            self.format_type(element_type)?;
            first = false;
        }
        write!(self.writer, ")")?;
        Ok(())
    }

    fn format_dyn_trait(&mut self, dyn_trait: nojson::RawJsonValue) -> crate::Result<()> {
        write!(self.writer, "dyn ")?;

        let traits = dyn_trait.to_member("traits")?.required()?;
        let mut first = true;

        for trait_obj in traits.to_array()? {
            if !first {
                write!(self.writer, " + ")?;
            }

            let trait_info = trait_obj.to_member("trait")?.required()?;
            let trait_path = trait_info
                .to_member("path")?
                .required()?
                .to_unquoted_string_str()?;

            write!(self.writer, "{}", trait_path)?;

            // Handle generic args if present
            if let Some(args) = trait_info.to_member("args")?.get()
                && !args.kind().is_null()
            {
                let Some(angle_bracketed) = args.to_member("angle_bracketed")?.get() else {
                    first = false;
                    continue;
                };

                self.format_angle_bracketed_args(angle_bracketed)?;
            }

            first = false;
        }

        // Add lifetime if present
        if let Some(lifetime) = dyn_trait.to_member("lifetime")?.get()
            && !lifetime.kind().is_null() {
                let lifetime_str = lifetime.to_unquoted_string_str()?;
                write!(self.writer, " + {}", lifetime_str)?;
            }

        Ok(())
    }

    fn format_impl_trait(&mut self, impl_trait: nojson::RawJsonValue) -> crate::Result<()> {
        write!(self.writer, "impl ")?;

        let mut first = true;
        for trait_obj in impl_trait.to_array()? {
            if !first {
                write!(self.writer, " + ")?;
            }

            // Handle lifetime bounds (e.g., 'outlives': "'_")
            if let Some(outlives) = trait_obj.to_member("outlives")?.get() {
                let lifetime_str = outlives.to_unquoted_string_str()?;
                write!(self.writer, "{}", lifetime_str)?;
                first = false;
                continue;
            }

            // Handle trait bounds
            let trait_bound = trait_obj.to_member("trait_bound")?.required()?;
            let trait_info = trait_bound.to_member("trait")?.required()?;
            let trait_path = trait_info
                .to_member("path")?
                .required()?
                .to_unquoted_string_str()?;

            write!(self.writer, "{}", trait_path)?;

            // Handle generic args and constraints
            if let Some(args) = trait_info.to_member("args")?.get()
                && !args.kind().is_null()
            {
                self.format_impl_trait_args(args)?;
            }

            first = false;
        }

        Ok(())
    }

    fn format_impl_trait_args(&mut self, args: nojson::RawJsonValue) -> crate::Result<()> {
        if let Some(angle_bracketed) = args.to_member("angle_bracketed")?.get() {
            write!(self.writer, "<")?;

            let args_array = angle_bracketed.to_member("args")?;
            let mut has_args = false;
            if let Some(args_list) = args_array.get() {
                let mut first = true;
                for arg in args_list.to_array()? {
                    if !first {
                        write!(self.writer, ", ")?;
                    }

                    if let Some(arg_type) = arg.to_member("type")?.get() {
                        self.format_type(arg_type)?;
                    }

                    first = false;
                    has_args = true;
                }
            }

            let constraints = angle_bracketed.to_member("constraints")?;
            if let Some(constraints_list) = constraints.get() {
                let mut first = true;
                for constraint in constraints_list.to_array()? {
                    if has_args || !first {
                        write!(self.writer, ", ")?;
                    }

                    let name = constraint
                        .to_member("name")?
                        .required()?
                        .to_unquoted_string_str()?;
                    write!(self.writer, "{}", name)?;

                    if let Some(binding) = constraint.to_member("binding")?.get()
                        && let Some(equality) = binding.to_member("equality")?.get() {
                            write!(self.writer, " = ")?;
                            self.format_type(equality.to_member("type")?.required()?)?;
                        }

                    first = false;
                }
            }

            write!(self.writer, ">")?;
        }

        Ok(())
    }

    fn format_slice(&mut self, slice: nojson::RawJsonValue) -> crate::Result<()> {
        write!(self.writer, "[")?;
        self.format_type(slice)?;
        write!(self.writer, "]")?;
        Ok(())
    }

    fn format_array(&mut self, array: nojson::RawJsonValue) -> crate::Result<()> {
        let inner_type = array.to_member("type")?.required()?;
        let len = array
            .to_member("len")?
            .required()?
            .to_unquoted_string_str()?;

        write!(self.writer, "[")?;
        self.format_type(inner_type)?;
        write!(self.writer, "; {}]", len)?;
        Ok(())
    }

    fn format_function_pointer(&mut self, fn_ptr: nojson::RawJsonValue) -> crate::Result<()> {
        let sig = fn_ptr.to_member("sig")?.required()?;

        // Format function pointer signature
        write!(self.writer, "fn(")?;

        // Format inputs
        let inputs = sig.to_member("inputs")?.required()?;
        for (i, input_pair) in inputs.to_array()?.enumerate() {
            if i > 0 {
                write!(self.writer, ", ")?;
            }

            let input_array: Vec<_> = input_pair.to_array()?.collect();
            if input_array.len() >= 2 {
                let param_type = input_array[1];
                self.format_type(param_type)?;
            }
        }

        write!(self.writer, ")")?;

        // Format output
        let output = sig.to_member("output")?;
        if let Some(output_type) = output.get()
            && !output_type.kind().is_null() {
                write!(self.writer, " -> ")?;
                self.format_type(output_type)?;
            }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_generic() -> crate::Result<()> {
        assert_format(r#"{"generic":"Self"}"#, "Self")
    }

    #[test]
    fn format_primitive() -> crate::Result<()> {
        assert_format(r#"{"primitive":"u32"}"#, "u32")
    }

    #[test]
    fn format_primitive_string() -> crate::Result<()> {
        assert_format(r#"{"primitive":"str"}"#, "str")
    }

    #[test]
    fn format_resolved_path_simple() -> crate::Result<()> {
        assert_format(
            r#"{"resolved_path":{"path":"std::vec::Vec","args":null}}"#,
            "std::vec::Vec",
        )
    }

    #[test]
    fn format_resolved_path_with_generic_args() -> crate::Result<()> {
        assert_format(
            r#"{"resolved_path":{"path":"Vec","args":{"angle_bracketed":{"args":[{"type":{"primitive":"i32"}}]}}}}"#,
            "Vec<i32>",
        )
    }

    #[test]
    fn format_resolved_path_with_multiple_args() -> crate::Result<()> {
        assert_format(
            r#"{"resolved_path":{"path":"HashMap","args":{"angle_bracketed":{"args":[{"type":{"primitive":"String"}},{"type":{"primitive":"i32"}}]}}}}"#,
            "HashMap<String, i32>",
        )
    }

    #[test]
    fn format_borrowed_ref() -> crate::Result<()> {
        assert_format(
            r#"{"borrowed_ref":{"lifetime":null,"is_mutable":false,"type":{"primitive":"str"}}}"#,
            "&str",
        )
    }

    #[test]
    fn format_mutable_borrowed_ref() -> crate::Result<()> {
        assert_format(
            r#"{"borrowed_ref":{"lifetime":null,"is_mutable":true,"type":{"primitive":"Vec"}}}"#,
            "&mut Vec",
        )
    }

    #[test]
    fn format_raw_pointer_const() -> crate::Result<()> {
        assert_format(
            r#"{"raw_pointer":{"is_mutable":false,"type":{"resolved_path":{"path":"objc_selector","id":12403,"args":null}}}}"#,
            "*const objc_selector",
        )
    }

    #[test]
    fn format_raw_pointer_mut() -> crate::Result<()> {
        assert_format(
            r#"{"raw_pointer":{"is_mutable":true,"type":{"resolved_path":{"path":"objc_selector","id":12403,"args":null}}}}"#,
            "*mut objc_selector",
        )
    }

    #[test]
    fn format_qualified_path() -> crate::Result<()> {
        assert_format(
            r#"{"qualified_path":{"name":"AtomicInner","args":null,"self_type":{"generic":"T"},"trait":{"path":"AtomicPrimitive","id":27989,"args":null}}}"#,
            "<T as AtomicPrimitive>::AtomicInner",
        )
    }

    #[test]
    fn format_tuple_empty() -> crate::Result<()> {
        assert_format(r#"{"tuple":[]}"#, "()")
    }

    #[test]
    fn format_tuple_single_element() -> crate::Result<()> {
        assert_format(r#"{"tuple":[{"primitive":"i32"}]}"#, "(i32)")
    }

    #[test]
    fn format_tuple_multiple_elements() -> crate::Result<()> {
        assert_format(
            r#"{"tuple":[{"primitive":"i32"},{"primitive":"str"},{"primitive":"bool"}]}"#,
            "(i32, str, bool)",
        )
    }

    #[test]
    fn format_nested_generic() -> crate::Result<()> {
        assert_format(
            r#"{"resolved_path":{"path":"Option","args":{"angle_bracketed":{"args":[{"type":{"resolved_path":{"path":"Vec","args":{"angle_bracketed":{"args":[{"type":{"primitive":"u8"}}]}}}}}]}}}}"#,
            "Option<Vec<u8>>",
        )
    }

    #[test]
    fn format_borrowed_ref_complex_type() -> crate::Result<()> {
        assert_format(
            r#"{"borrowed_ref":{"lifetime":null,"is_mutable":false,"type":{"resolved_path":{"path":"Vec","args":{"angle_bracketed":{"args":[{"type":{"primitive":"String"}}]}}}}}}"#,
            "&Vec<String>",
        )
    }

    #[test]
    fn format_mutable_borrowed_ref_with_generics() -> crate::Result<()> {
        assert_format(
            r#"{"borrowed_ref":{"lifetime":null,"is_mutable":true,"type":{"resolved_path":{"path":"Vec","args":{"angle_bracketed":{"args":[{"type":{"primitive":"i32"}}]}}}}}}"#,
            "&mut Vec<i32>",
        )
    }

    #[test]
    fn format_tuple_with_complex_types() -> crate::Result<()> {
        assert_format(
            r#"{"tuple":[{"borrowed_ref":{"lifetime":null,"is_mutable":false,"type":{"primitive":"str"}}},{"resolved_path":{"path":"Vec","args":{"angle_bracketed":{"args":[{"type":{"primitive":"i32"}}]}}}}]}"#,
            "(&str, Vec<i32>)",
        )
    }

    #[test]
    fn format_dyn_trait_multiple_bounds() -> crate::Result<()> {
        assert_format(
            r#"{"dyn_trait":{"traits":[{"trait":{"path":"Any","id":415,"args":null},"generic_params":[]},{"trait":{"path":"Send","id":6,"args":null},"generic_params":[]}],"lifetime":"'static"}}"#,
            "dyn Any + Send + 'static",
        )
    }

    #[test]
    fn format_dyn_trait_single_bound() -> crate::Result<()> {
        assert_format(
            r#"{"dyn_trait":{"traits":[{"trait":{"path":"Display","id":123,"args":null},"generic_params":[]}],"lifetime":null}}"#,
            "dyn Display",
        )
    }

    #[test]
    fn format_dyn_trait_with_lifetime() -> crate::Result<()> {
        assert_format(
            r#"{"dyn_trait":{"traits":[{"trait":{"path":"Iterator","id":200,"args":null},"generic_params":[]}],"lifetime":"'a"}}"#,
            "dyn Iterator + 'a",
        )
    }

    #[test]
    fn format_impl_trait_simple() -> crate::Result<()> {
        assert_format(
            r#"{"impl_trait":[{"trait_bound":{"trait":{"path":"Iterator","id":474,"args":null},"generic_params":[],"modifier":"none"}}]}"#,
            "impl Iterator",
        )
    }

    #[test]
    fn format_impl_trait_with_associated_type() -> crate::Result<()> {
        assert_format(
            r#"{"impl_trait":[{"trait_bound":{"trait":{"path":"Iterator","id":474,"args":{"angle_bracketed":{"args":[],"constraints":[{"name":"Item","args":null,"binding":{"equality":{"type":{"generic":"Self"}}}}]}}}, "generic_params":[],"modifier":"none"}}]}"#,
            "impl Iterator<Item = Self>",
        )
    }

    #[test]
    fn format_impl_trait_multiple_bounds() -> crate::Result<()> {
        assert_format(
            r#"{"impl_trait":[{"trait_bound":{"trait":{"path":"Iterator","id":474,"args":null},"generic_params":[],"modifier":"none"}},{"trait_bound":{"trait":{"path":"Send","id":6,"args":null},"generic_params":[],"modifier":"none"}}]}"#,
            "impl Iterator + Send",
        )
    }

    #[test]
    fn format_impl_trait_with_lifetime() -> crate::Result<()> {
        assert_format(
            r#"{"impl_trait":[{"outlives":"'_"},{"trait_bound":{"trait":{"path":"Iterator","id":147,"args":{"angle_bracketed":{"args":[],"constraints":[{"name":"Item","args":null,"binding":{"equality":{"type":{"tuple":[{"primitive":"usize"},{"borrowed_ref":{"lifetime":null,"is_mutable":false,"type":{"primitive":"str"}}}]}}}}]}}}, "generic_params":[],"modifier":"none"}}]}"#,
            "impl '_ + Iterator<Item = (usize, &str)>",
        )
    }

    #[test]
    fn format_slice() -> crate::Result<()> {
        assert_format(r#"{"slice":{"primitive":"u8"}}"#, "[u8]")
    }

    #[test]
    fn format_array() -> crate::Result<()> {
        assert_format(r#"{"array":{"type":{"generic":"T"},"len":"N"}}"#, "[T; N]")
    }

    #[test]
    fn format_array_with_resolved_path() -> crate::Result<()> {
        assert_format(
            r#"{"array":{"type":{"resolved_path":{"path":"String","args":null}},"len":"32"}}"#,
            "[String; 32]",
        )
    }

    #[test]
    fn format_function_pointer() -> crate::Result<()> {
        assert_format(
            r#"{"function_pointer":{"sig":{"inputs":[["-",{"resolved_path":{"path":"Layout","id":9530,"args":null}}]],"output":null,"is_c_variadic":false},"generic_params":[],"header":{"is_const":false,"is_unsafe":false,"is_async":false,"abi":"Rust"}}}"#,
            "fn(Layout)",
        )
    }

    #[test]
    fn format_function_pointer_with_return() -> crate::Result<()> {
        assert_format(
            r#"{"function_pointer":{"sig":{"inputs":[["-",{"primitive":"i32"}]],"output":{"primitive":"i32"},"is_c_variadic":false},"generic_params":[],"header":{"is_const":false,"is_unsafe":false,"is_async":false,"abi":"Rust"}}}"#,
            "fn(i32) -> i32",
        )
    }

    fn assert_format(input: &str, expected: &str) -> crate::Result<()> {
        let doc = empty_doc();
        let raw_json = nojson::RawJson::parse(input)?;
        let formatted = format_to_string(&doc, raw_json.value())?;
        assert_eq!(formatted, expected);
        Ok(())
    }

    fn empty_doc() -> crate::doc::CrateDoc {
        let text = r#"{"root": 0, "index": {"0": {"name": "test", "visibility": "public", "inner": {"module": {"items": []}}, "docs": null, "deprecation": null}}}"#;
        crate::doc::CrateDoc::parse(std::path::PathBuf::from(""), text).expect("bug")
    }
}
