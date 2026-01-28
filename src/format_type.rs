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
        } else if let Some(borrowed_ref) = ty.to_member("borrowed_ref")?.get()
            && borrowed_ref
                .to_member("lifetime")?
                .required()?
                .kind()
                .is_null()
        {
            self.format_borrowed_ref(borrowed_ref)
        } else if let Some(raw_pointer) = ty.to_member("raw_pointer")?.get() {
            self.format_raw_pointer(raw_pointer)
        } else if let Some(qualified_path) = ty.to_member("qualified_path")?.get() {
            self.format_qualified_path(qualified_path)
        } else if let Some(tuple) = ty.to_member("tuple")?.get() {
            self.format_tuple(tuple)
        } else if let Some(dyn_trait) = ty.to_member("dyn_trait")?.get() {
            self.format_dyn_trait(dyn_trait)
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
        self.format_prefixed_type(borrowed_ref, "&mut ", "&")
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
        if let Some(lifetime) = dyn_trait.to_member("lifetime")?.get() {
            if !lifetime.kind().is_null() {
                let lifetime_str = lifetime.to_unquoted_string_str()?;
                write!(self.writer, " + {}", lifetime_str)?;
            }
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
