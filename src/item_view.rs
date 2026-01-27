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
}
