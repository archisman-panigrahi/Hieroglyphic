include!(concat!(env!("OUT_DIR"), "/symbol_table.rs"));

// Original code from:
// https://github.com/FineFindus/detexify-rust/blob/311002feb0519f483ef1f9cc8206648286128ff5/src/symbol.rs

/// LateX Symbol
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// Command to display the symbol.
    pub command: &'static str,
    /// Package which the symbol belongs to.
    pub package: &'static str,
    /// Font encoding used for the symbol.
    font_encoding: &'static str,
    /// Whether the symbol is available in text mode.
    pub text_mode: bool,
    /// Whether the symbol is available in math mode.
    pub math_mode: bool,
}

impl Symbol {
    /// Returns the symbol that the `id` specifies.
    pub fn from_id(id: &str) -> Option<Self> {
        SYMBOL_TABLE.get(id).cloned()
    }

    /// Returns the `id` of the symbol.
    pub fn id(&self) -> &'static str {
        let id = format!(
            "{}-{}-{}",
            self.package,
            self.font_encoding,
            self.command.replace('\\', "_")
        );
        let key = base32::encode(base32::Alphabet::Rfc4648 { padding: false }, id.as_bytes());
        // SAFETY: safe to unwrap, since key must be valid, as it is only possible to get a Symbol
        // from the symbol table
        SYMBOL_TABLE.get_key(&key).unwrap()
    }
}

pub fn iter_symbols() -> impl Iterator<Item = Symbol> {
    SYMBOL_TABLE.values().cloned()
}

#[cfg(test)]
mod tests {

    use super::iter_symbols;
    use super::Symbol;

    #[test]
    fn test_from_id() {
        let symbol = Symbol::from_id("NRQXIZLYGJSS2T2UGEWV65DFPB2GC43DNFUWG2LSMN2W2");

        assert_eq!(
            symbol,
            Some(Symbol {
                command: "\\textasciicircum",
                package: "latex2e",
                font_encoding: "OT1",
                text_mode: true,
                math_mode: false
            })
        );
    }

    #[test]
    fn test_iterate_symbols() {
        assert_eq!(iter_symbols().count(), 1098);
    }

    #[test]
    fn test_id_get_id() {
        for symbol in iter_symbols() {
            assert_eq!(Symbol::from_id(symbol.id()).unwrap(), symbol);
        }
    }
}
