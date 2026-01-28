pub fn format_function_to_string(
    doc: &crate::doc::CrateDoc,
    item: &crate::doc::Item,
) -> crate::Result<String> {
    todo!()
}

#[derive(Debug)]
pub struct FunctionFormatter<'a> {
    // todo: add writer
    doc: &'a crate::doc::CrateDoc,
    item: &'a crate::doc::Item,
}

#[cfg(test)]
mod tests {
    use super::*;

    // todo: test
    //  {"sig":{"inputs":[["key",{"generic":"K"}]],"output":{"resolved_path":{"path":"Result","id":406,"args":{"angle_bracketed":{"args":[{"type":{"resolved_path":{"path":"String","id":447,"args":null}}},{"type":{"resolved_path":{"path":"std::env::VarError","id":628,"args":null}}}],"constraints":[]}}}},"is_c_variadic":false},"generics":{"params":[{"name":"K","kind":{"type":{"bounds":[{"trait_bound":{"trait":{"path":"AsRef","id":629,"args":{"angle_bracketed":{"args":[{"type":{"resolved_path":{"path":"std::ffi::OsStr","id":630,"args":null}}}],"constraints":[]}}},"generic_params":[],"modifier":"none"}},{"trait_bound":{"trait":{"path":"AsRef","id":629,"args":{"angle_bracketed":{"args":[{"type":{"primitive":"str"}}],"constraints":[]}}},"generic_params":[],"modifier":"none"}}],"default":null,"is_synthetic":false}}}],"where_predicates":[]},"header":{"is_const":false,"is_unsafe":false,"is_async":false,"abi":"Rust"},"has_body":true}
}
