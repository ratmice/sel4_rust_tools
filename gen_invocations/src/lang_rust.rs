const COMMON_HEADER: &str = r#"
// TODO 
const _: () = ();
"#;

pub const INVOCATION_TEMPLATE: &str = const_format::concatcp!(COMMON_HEADER, r#""#);
pub const SEL4_ARCH_INVOCATION_TEMPLATE: &str = const_format::concatcp!(COMMON_HEADER, r#""#);
pub const ARCH_INVOCATION_TEMPLATE: &str = const_format::concatcp!(COMMON_HEADER, r#""#);
