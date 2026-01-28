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
        } else if let Some(tuple) = ty.to_member("tuple")?.get() {
            self.format_tuple(tuple)
        } else {
            write!(self.writer, "{}", ty)?;
            Ok(())
        }
    }

    fn format_generic(&mut self, generic: nojson::RawJsonValue) -> crate::Result<()> {
        let formatted: String = generic.try_into()?;
        write!(self.writer, "{}", formatted)?;
        Ok(())
    }

    fn format_resolved_path(&mut self, resolved: nojson::RawJsonValue) -> crate::Result<()> {
        let path: String = resolved.to_member("path")?.required()?.try_into()?;

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
        path: String,
        args: nojson::RawJsonValue,
    ) -> crate::Result<()> {
        let mut formatted_args = Vec::new();

        if let Some(angle_bracketed) = args.to_member("angle_bracketed")?.get() {
            let args_array = angle_bracketed.to_member("args")?.required()?;

            for arg in args_array.to_array()? {
                if let Some(arg_type) = arg.to_member("type")?.get() {
                    let mut buffer = Vec::new();
                    let mut temp_formatter = TypeFormatter::new(&mut buffer, self.doc);
                    temp_formatter.format_type(arg_type)?;
                    formatted_args.push(String::from_utf8(buffer).expect("bug"));
                } else if let Some(lifetime) = arg.to_member("lifetime")?.get() {
                    let lifetime_str: String = lifetime.try_into()?;
                    formatted_args.push(lifetime_str);
                }
            }
        }

        if !formatted_args.is_empty() {
            write!(self.writer, "{}<{}>", path, formatted_args.join(", "))?;
        } else {
            write!(self.writer, "{}", path)?;
        }
        Ok(())
    }

    fn format_primitive(&mut self, primitive: nojson::RawJsonValue) -> crate::Result<()> {
        let formatted: String = primitive.try_into()?;
        write!(self.writer, "{}", formatted)?;
        Ok(())
    }

    fn format_borrowed_ref(&mut self, borrowed_ref: nojson::RawJsonValue) -> crate::Result<()> {
        let is_mutable: bool = borrowed_ref
            .to_member("is_mutable")?
            .required()?
            .try_into()?;
        let inner_type = borrowed_ref.to_member("type")?.required()?;
        let prefix = if is_mutable { "&mut " } else { "&" };
        write!(self.writer, "{}", prefix)?;
        self.format_type(inner_type)
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
}
