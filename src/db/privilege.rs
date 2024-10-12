pub trait Privilege {
    fn grantor(&self) -> &str;
    fn grantee(&self) -> &str;
    fn privilege_type(&self) -> &str;
}
