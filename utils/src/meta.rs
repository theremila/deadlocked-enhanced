//! linker metadata helpers for embedding named byte sections.

/// embeds a byte slice as a named linker section symbol.
///
/// # usage
///
/// ```
/// use utils::embed_metadata;
///
/// embed_metadata!(VERSION, ".version", &[50]);
/// ```
#[macro_export]
macro_rules! embed_metadata {
    ($name:ident, $section:literal, $value:expr) => {
        #[used]
        #[unsafe(link_section = $section)]
        #[unsafe(no_mangle)]
        static $name: [u8; $value.len()] = *$value;
    };
}
