pub fn format_enum_variant_to_string(
    doc: &crate::doc::CrateDoc,
    item: &crate::doc::Item,
) -> crate::Result<String> {
    let name = item.name.as_ref().expect("bug");
    let inner = item.inner(&doc.json);
    let mut buffer = Vec::new();
    let mut formatter = EnumVariantFormatter::new(&mut buffer, doc, name);
    formatter.format(inner)?;
    Ok(String::from_utf8(buffer).expect("bug"))
}

#[derive(Debug)]
pub struct EnumVariantFormatter<'a, W> {
    writer: W,
    doc: &'a crate::doc::CrateDoc,
    name: &'a str,
}

impl<'a, W: std::io::Write> EnumVariantFormatter<'a, W> {
    pub fn new(writer: W, doc: &'a crate::doc::CrateDoc, name: &'a str) -> Self {
        Self { writer, doc, name }
    }

    pub fn format(&mut self, inner: nojson::RawJsonValue) -> crate::Result<()> {
        // Write variant name
        write!(self.writer, "{}", self.name)?;

        // Check for discriminant
        let discriminant = inner.to_member("discriminant")?;
        if let Some(disc) = discriminant.get() {
            if !disc.kind().is_null() {
                write!(self.writer, " = ")?;
                let disc_str: String = disc.try_into()?;
                write!(self.writer, "{}", disc_str)?;
            }
        }

        // Format variant kind (struct, tuple, or unit)
        let kind = inner.to_member("kind")?;
        if let Some(kind_obj) = kind.get() {
            // Only process if kind is an object, not a string
            if !kind_obj.kind().is_string() {
                // Check for struct variant
                if let Some(struct_kind) = kind_obj.to_member("struct")?.get() {
                    self.format_struct_variant(struct_kind)?;
                }
                // Check for tuple variant
                else if let Some(tuple_kind) = kind_obj.to_member("tuple")?.get() {
                    self.format_tuple_variant(tuple_kind)?;
                }
                // Unit variant has no additional formatting
            }
            // If kind is a string (like "plain"), it's a unit variant with no additional formatting
        }

        Ok(())
    }

    fn format_struct_variant(&mut self, struct_obj: nojson::RawJsonValue) -> crate::Result<()> {
        let fields = struct_obj.to_member("fields")?.required()?;
        let field_ids: Vec<_> = fields.to_array()?.collect();

        write!(self.writer, " {{ ")?;

        for (i, field_id_value) in field_ids.iter().enumerate() {
            if i > 0 {
                write!(self.writer, ", ")?;
            }

            let field_item_value = self.doc.items.get(&self.doc.json, *field_id_value)?;
            let field_item = crate::doc::Item::try_from(field_item_value)?;
            let field_name = field_item.name.as_deref().unwrap_or("?");
            let field_inner = field_item.inner(&self.doc.json);
            // Use the entire field_inner as the type, not field_inner.to_member("type")
            let formatted_type = crate::format_type::format_to_string(self.doc, field_inner)?;

            write!(self.writer, "{}: ", field_name)?;
            write!(self.writer, "{}", formatted_type)?;
        }

        write!(self.writer, " }}")?;
        Ok(())
    }

    fn format_tuple_variant(&mut self, tuple_obj: nojson::RawJsonValue) -> crate::Result<()> {
        let fields = tuple_obj.to_member("fields")?.required()?;
        let field_ids: Vec<_> = fields.to_array()?.collect();

        write!(self.writer, "(")?;

        for (i, field_id_value) in field_ids.iter().enumerate() {
            if i > 0 {
                write!(self.writer, ", ")?;
            }

            let field_item_value = self.doc.items.get(&self.doc.json, *field_id_value)?;
            let field_item = crate::doc::Item::try_from(field_item_value)?;
            let field_inner = field_item.inner(&self.doc.json);
            // Use the entire field_inner as the type
            let formatted_type = crate::format_type::format_to_string(self.doc, field_inner)?;

            write!(self.writer, "{}", formatted_type)?;
        }

        write!(self.writer, ")")?;
        Ok(())
    }
}

pub fn format_function_to_string(
    doc: &crate::doc::CrateDoc,
    name: &str,
    inner: nojson::RawJsonValue,
) -> crate::Result<String> {
    let mut buffer = Vec::new();
    let mut formatter = FunctionFormatter::new(&mut buffer, doc, name);
    formatter.format(inner)?;
    Ok(String::from_utf8(buffer).expect("bug"))
}

#[derive(Debug)]
pub struct FunctionFormatter<'a, W> {
    writer: W,
    doc: &'a crate::doc::CrateDoc,
    name: &'a str,
}

impl<'a, W: std::io::Write> FunctionFormatter<'a, W> {
    pub fn new(writer: W, doc: &'a crate::doc::CrateDoc, name: &'a str) -> Self {
        Self { writer, doc, name }
    }

    pub fn format(&mut self, inner: nojson::RawJsonValue) -> crate::Result<()> {
        // println!("{inner}"); // TODO
        self.format_function_signature(inner)?;
        Ok(())
    }

    fn format_function_signature(&mut self, function: nojson::RawJsonValue) -> crate::Result<()> {
        let sig = function.to_member("sig")?.required()?;

        // Format header (const, unsafe, async)
        self.format_function_header(function)?;

        // Format function name
        write!(self.writer, "fn {}", self.name)?;

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

    fn format_function_header(&mut self, function: nojson::RawJsonValue) -> crate::Result<()> {
        let header = function.to_member("header")?;

        if let Some(header) = header.get() {
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
        // Extract bound_predicate wrapper
        if let Some(bound_predicate) = predicate.to_member("bound_predicate")?.get() {
            // Extract the type being constrained
            if let Some(lhs) = bound_predicate.to_member("type")?.get() {
                let formatted_lhs = crate::format_type::format_to_string(self.doc, lhs)?;
                write!(self.writer, "{}", formatted_lhs)?;
            }

            // Format the bounds
            let bounds = bound_predicate.to_member("bounds")?;
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

                            if trait_path.is_empty() {
                                // Handle empty path (associated types)
                                write!(self.writer, "?")?;
                            } else {
                                write!(self.writer, "{}", trait_path)?;
                            }

                            if let Some(args) = trait_info.to_member("args")?.get() {
                                if !args.kind().is_null() {
                                    self.format_trait_args(args)?;
                                }
                            }
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

    #[test]
    fn format_function_with_generic_trait_bounds() -> crate::Result<()> {
        let doc = empty_doc();
        let json_str = r#"{
                "sig": {
                    "inputs": [["key", {"generic": "K"}]],
                    "output": {
                        "resolved_path": {
                            "path": "Result",
                            "id": 406,
                            "args": {
                                "angle_bracketed": {
                                    "args": [
                                        {"type": {"resolved_path": {"path": "String", "id": 447, "args": null}}},
                                        {"type": {"resolved_path": {"path": "std::env::VarError", "id": 628, "args": null}}}
                                    ],
                                    "constraints": []
                                }
                            }
                        }
                    },
                    "is_c_variadic": false,
                    "header": {
                        "is_const": false,
                        "is_unsafe": false,
                        "is_async": false,
                        "abi": "Rust"
                    }
                },
                "generics": {
                    "params": [{
                        "name": "K",
                        "kind": {
                            "type": {
                                "bounds": [
                                    {
                                        "trait_bound": {
                                            "trait": {
                                                "path": "AsRef",
                                                "id": 629,
                                                "args": {
                                                    "angle_bracketed": {
                                                        "args": [{"type": {"resolved_path": {"path": "std::ffi::OsStr", "id": 630, "args": null}}}],
                                                        "constraints": []
                                                    }
                                                }
                                            },
                                            "generic_params": [],
                                            "modifier": "none"
                                        }
                                    },
                                    {
                                        "trait_bound": {
                                            "trait": {
                                                "path": "AsRef",
                                                "id": 629,
                                                "args": {
                                                    "angle_bracketed": {
                                                        "args": [{"type": {"primitive": "str"}}],
                                                        "constraints": []
                                                    }
                                                }
                                            },
                                            "generic_params": [],
                                            "modifier": "none"
                                        }
                                    }
                                ],
                                "default": null,
                                "is_synthetic": false
                            }
                        }
                    }],
                    "where_predicates": []
                },
                "has_body": true
            }"#;

        let raw_json = nojson::RawJson::parse(json_str)?;
        let formatted = format_function_to_string(&doc, "var", raw_json.value())?;

        assert_eq!(
            formatted,
            "fn var<K: AsRef<std::ffi::OsStr> + AsRef<str>>(key: K) -> Result<String, std::env::VarError>"
        );

        Ok(())
    }

    #[test]
    fn format_function_header_with_modifiers() -> crate::Result<()> {
        let doc = empty_doc();
        let json_str = r#"{"sig": {"inputs": [], "output": null, "is_c_variadic": false, "header": {"is_const": true, "is_unsafe": true, "is_async": false, "abi": "Rust"}}, "generics": {"params": [], "where_predicates": []}, "has_body": true}"#;

        let raw_json = nojson::RawJson::parse(json_str)?;
        let sig = raw_json.value().to_member("sig")?.required()?;

        let mut buffer = Vec::new();
        let mut formatter = FunctionFormatter::new(&mut buffer, &doc, "test_fn");
        formatter.format_function_header(sig)?;

        let result = String::from_utf8_lossy(&buffer);
        assert_eq!(result, "const unsafe ");

        Ok(())
    }

    #[test]
    fn format_function_header_without_modifiers() -> crate::Result<()> {
        let doc = empty_doc();
        let json_str = r#"{"sig": {"inputs": [], "output": null, "is_c_variadic": false, "header": {"is_const": false, "is_unsafe": false, "is_async": false, "abi": "Rust"}}, "generics": {"params": [], "where_predicates": []}, "has_body": true}"#;

        let raw_json = nojson::RawJson::parse(json_str)?;
        let sig = raw_json.value().to_member("sig")?.required()?;

        let mut buffer = Vec::new();
        let mut formatter = FunctionFormatter::new(&mut buffer, &doc, "test_fn");
        formatter.format_function_header(sig)?;

        let result = String::from_utf8_lossy(&buffer);
        assert_eq!(result, "");

        Ok(())
    }

    #[test]
    fn format_function_header_async() -> crate::Result<()> {
        let doc = empty_doc();
        let json_str = r#"{"sig": {"inputs": [], "output": null, "is_c_variadic": false, "header": {"is_const": false, "is_unsafe": false, "is_async": true, "abi": "Rust"}}, "generics": {"params": [], "where_predicates": []}, "has_body": true}"#;

        let raw_json = nojson::RawJson::parse(json_str)?;
        let sig = raw_json.value().to_member("sig")?.required()?;

        let mut buffer = Vec::new();
        let mut formatter = FunctionFormatter::new(&mut buffer, &doc, "test_fn");
        formatter.format_function_header(sig)?;

        let result = String::from_utf8_lossy(&buffer);
        assert_eq!(result, "async ");

        Ok(())
    }

    #[test]
    fn format_function_header_missing() -> crate::Result<()> {
        let doc = empty_doc();
        let json_str = r#"{"sig": {"inputs": [], "output": null, "is_c_variadic": false}, "generics": {"params": [], "where_predicates": []}, "has_body": true}"#;

        let raw_json = nojson::RawJson::parse(json_str)?;
        let sig = raw_json.value().to_member("sig")?.required()?;

        let mut buffer = Vec::new();
        let mut formatter = FunctionFormatter::new(&mut buffer, &doc, "test_fn");
        // Should not panic when header is missing
        formatter.format_function_header(sig)?;

        let result = String::from_utf8_lossy(&buffer);
        assert_eq!(result, "");

        Ok(())
    }

    #[test]
    fn format_function_header_const_and_unsafe() -> crate::Result<()> {
        let doc = empty_doc();
        let json_str = r#"{"sig": {"inputs": [], "output": null, "is_c_variadic": false, "header": {"is_const": true, "is_unsafe": false, "is_async": false, "abi": "Rust"}}, "generics": {"params": [], "where_predicates": []}, "has_body": true}"#;

        let raw_json = nojson::RawJson::parse(json_str)?;
        let sig = raw_json.value().to_member("sig")?.required()?;

        let mut buffer = Vec::new();
        let mut formatter = FunctionFormatter::new(&mut buffer, &doc, "test_fn");
        formatter.format_function_header(sig)?;

        let result = String::from_utf8_lossy(&buffer);
        assert_eq!(result, "const ");

        Ok(())
    }

    #[test]
    fn format_where_clause_with_trait_bounds() -> crate::Result<()> {
        let doc = empty_doc();
        let json_str = r#"{
        "sig": {
            "inputs": [["self", {"borrowed_ref": {"lifetime": null, "is_mutable": true, "type": {"generic": "Self"}}}]],
            "output": {"generic": "Option"},
            "is_c_variadic": false,
            "header": {"is_const": false, "is_unsafe": false, "is_async": false, "abi": "Rust"}
        },
        "generics": {
            "params": [{"name": "B", "kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}}],
            "where_predicates": [
                {
                    "bound_predicate": {
                        "type": {"generic": "Self"},
                        "bounds": [{"trait_bound": {"trait": {"path": "Sized", "id": 12, "args": null}, "generic_params": [], "modifier": "none"}}],
                        "generic_params": []
                    }
                },
                {
                    "bound_predicate": {
                        "type": {"generic": "B"},
                        "bounds": [{"trait_bound": {"trait": {"path": "Default", "id": 13, "args": null}, "generic_params": [], "modifier": "none"}}],
                        "generic_params": []
                    }
                }
            ]
        },
        "has_body": true
    }"#;

        let raw_json = nojson::RawJson::parse(json_str)?;
        let formatted = format_function_to_string(&doc, "test_fn", raw_json.value())?;
        assert_eq!(
            formatted,
            "fn test_fn<B>(self: &mut Self) -> Option\nwhere\n    Self: Sized,\n    B: Default"
        );
        Ok(())
    }

    #[test]
    fn format_function_with_borrowed_ref_lifetime() -> crate::Result<()> {
        let doc = empty_doc();
        let json_str = r#"{
        "sig": {
            "inputs": [
                ["doc", {"borrowed_ref": {"lifetime": "'a", "is_mutable": false, "type": {"resolved_path": {"path": "crate::doc::CrateDoc", "id": 251, "args": null}}}}],
                ["item", {"borrowed_ref": {"lifetime": "'a", "is_mutable": false, "type": {"resolved_path": {"path": "crate::doc::Item", "id": 168, "args": null}}}}]
            ],
            "output": {"generic": "Self"},
            "is_c_variadic": false,
            "header": {"is_const": false, "is_unsafe": false, "is_async": false, "abi": "Rust"}
        },
        "generics": {"params": [], "where_predicates": []},
        "has_body": true
    }"#;

        let raw_json = nojson::RawJson::parse(json_str)?;
        let formatted = format_function_to_string(&doc, "new", raw_json.value())?;

        assert_eq!(
            formatted,
            "fn new(doc: &'a crate::doc::CrateDoc, item: &'a crate::doc::Item) -> Self"
        );

        Ok(())
    }

    #[test]
    fn format_function_with_impl_trait_return() -> crate::Result<()> {
        let doc = empty_doc();
        let json_str = r#"{
        "sig": {
            "inputs": [["self", {"generic": "Self"}]],
            "output": {
                "resolved_path": {
                    "path": "Result",
                    "id": 205,
                    "args": {
                        "angle_bracketed": {
                            "args": [
                                {
                                    "type": {
                                        "impl_trait": [
                                            {
                                                "trait_bound": {
                                                    "trait": {
                                                        "path": "Iterator",
                                                        "id": 474,
                                                        "args": {
                                                            "angle_bracketed": {
                                                                "args": [],
                                                                "constraints": [
                                                                    {
                                                                        "name": "Item",
                                                                        "args": null,
                                                                        "binding": {
                                                                            "equality": {
                                                                                "type": {"generic": "Self"}
                                                                            }
                                                                        }
                                                                    }
                                                                ]
                                                            }
                                                        }
                                                    },
                                                    "generic_params": [],
                                                    "modifier": "none"
                                                }
                                            }
                                        ]
                                    }
                                },
                                {
                                    "type": {
                                        "resolved_path": {
                                            "path": "JsonParseError",
                                            "id": 338,
                                            "args": null
                                        }
                                    }
                                }
                            ],
                            "constraints": []
                        }
                    }
                }
            },
            "is_c_variadic": false,
            "header": {
                "is_const": false,
                "is_unsafe": false,
                "is_async": false,
                "abi": "Rust"
            }
        },
        "generics": {"params": [], "where_predicates": []},
        "has_body": true
    }"#;

        let raw_json = nojson::RawJson::parse(json_str)?;
        let formatted = format_function_to_string(&doc, "into_iter", raw_json.value())?;

        assert_eq!(
            formatted,
            "fn into_iter(self: Self) -> Result<impl Iterator<Item = Self>, JsonParseError>"
        );

        Ok(())
    }

    #[test]
    fn format_function_with_const_modifier() -> crate::Result<()> {
        let doc = empty_doc();
        let json_str = r#"{"sig":{"inputs":[["name",{"borrowed_ref":{"lifetime":"'static","is_mutable":false,"type":{"primitive":"str"}}}]],"output":{"resolved_path":{"path":"OptSpec","id":411,"args":null}},"is_c_variadic":false},"generics":{"params":[],"where_predicates":[]},"header":{"is_const":true,"is_unsafe":false,"is_async":false,"abi":"Rust"},"has_body":true}"#;

        let raw_json = nojson::RawJson::parse(json_str)?;
        let formatted = format_function_to_string(&doc, "new_opt", raw_json.value())?;

        assert_eq!(formatted, "const fn new_opt(name: &'static str) -> OptSpec");

        Ok(())
    }

    fn empty_doc() -> crate::doc::CrateDoc {
        let text = r#"{"root": 0, "index": {"0": {"name": "test", "visibility": "public", "inner": {"module": {"items": []}}, "docs": null, "deprecation": null}}}"#;
        crate::doc::CrateDoc::parse(std::path::PathBuf::from(""), text).expect("bug")
    }
}
