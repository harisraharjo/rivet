// /// Describes how a sequence of token trees is delimited.
// /// Cannot use `proc_macro::Delimiter` directly because this
// /// structure should implement some additional traits.
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub enum Delimiter {
//     /// `( ... )`
//     Parenthesis,
//     /// `{ ... }`
//     Brace,
//     /// `[ ... ]`
//     Bracket,
//     // /// `∅ ... ∅`
//     // /// An invisible delimiter, that may, for example, appear around tokens coming from a
//     // /// "macro variable" `$var`. It is important to preserve operator priorities in cases like
//     // /// `$var * 3` where `$var` is `1 + 2`.
//     // /// Invisible delimiters might not survive roundtrip of a token stream through a string.
//     // Invisible(InvisibleOrigin),
// }

// // Note that the suffix is *not* considered when deciding the `LiteralType` in this
// // type. This means that float literals like `1f32` are classified by this type
// // as `Int`. Only upon conversion to `ast::LiteralType` will such a literal be
// // given the `Float` type.
// #[derive(Clone, Copy, PartialEq, Debug)]
// pub enum LiteralType {
//     Bool, // AST only, must never appear in a `Token`
//     Byte,
//     Char,
//     Integer, // e.g. `1`, `1u8`, `1f32`
//     // Float,   // e.g. `1.`, `1.0`, `1e3f32`
//     Str,
//     StrRaw(u8), // raw string delimited by `n` hash symbols
//     ByteStr,
//     // ByteStrRaw(u8), // raw byte string delimited by `n` hash symbols
//     // CStr,
//     // CStrRaw(u8),
//     // Err(ErrorGuaranteed),
// }

// /// A literal token.
// #[derive(Clone, Copy, PartialEq, Debug)]
// pub struct Lit {
//     pub ty: LiteralType,
//     // pub symbol: Symbol,
//     // pub suffix: Option<Symbol>,
// }

// // SAFETY: due to the `Clone` impl below, all fields of all variants other than
// // `Interpolated` must impl `Copy`.
// #[derive(PartialEq, Debug)]
// pub enum TokenType {
//     // BinOp(BinOpToken),
//     // BinOpEq(BinOpToken),

//     /* Structural symbols */
//     /// `.`
//     Dot,
//     /// `..=`
//     DotDotEq,
//     /// `,`
//     Comma,
//     /// `;`
//     Semi,
//     /// `:`
//     Colon,
//     /// Used by proc macros for representing lifetimes, not generated by lexer right now.
//     SingleQuote,
//     /// An opening delimiter (e.g., `{`).
//     OpenDelim(Delimiter),
//     /// A closing delimiter (e.g., `}`).
//     CloseDelim(Delimiter),

//     /* Literals */
//     Literal(Lit),

//     /// End Of File
//     Eof,
// }
