use isa::{instruction::Instruction, register::Register};
use logos::Logos;
use shared::{EnumCount, EnumVariants};
use thiserror::Error;

#[derive(Default, Debug)]
struct Span {
    start: usize,
    end: usize,
}

#[derive(Default, Debug)]
pub struct State {
    row: Span,
    column: Span,
    in_block_comments: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Error)]
pub enum LexingError {
    #[error("Invalid Integer at {0}")]
    InvalidInteger(usize),
    #[error("Non Ascii Character at {0}")]
    NonAsciiCharacter(usize),
    #[default]
    #[error("Unknown Syntax")]
    UnknownSyntax,
}

fn on_block_comment(lex: &mut logos::Lexer<Token>) -> logos::Skip {
    lex.extras.in_block_comments = !lex.extras.in_block_comments;
    logos::Skip
}

/// Update the line count and the char index.
fn on_newline(lex: &mut logos::Lexer<Token>) {
    lex.extras.row.start += 1;
    lex.extras.row.end = lex.span().end;
}

// const fn generate_mnemonic_re<'a>() -> &'a [u8] {
//     b"(?i:add|sub|and|or|xor|sll|srl|sra|)"
// }

//  let patterns = &[
//             br#"(?s)/\*.*?\*/"#,                     // Block comments (/* ... */)
//             br#"#.*"#,                                 // Line comments (# ...)
//             br#"(?i)\b(addi|add|lw|sw|beq)\b"#,       // Mnemonics (case-insensitive)
//             br#"\.(text|data|global|word)\b"#,         // Directives (.text, .data)
//             br#"x([0-9]{1,2})\b"#,                    // Registers (x0–x31)
//             br#"([a-zA-Z_][a-zA-Z0-9_]*):"#,           // Label definitions (loop:)
//             br#"(-?\d+)\b"#,                           // Immediates (e.g., -42)
//             br#"[(),]"#,                               // Punctuation
//             br#"\s+"#,                                 // Whitespace (ignored)
//         ];

#[derive(Debug, PartialEq)]
pub enum IdentifierType {
    Mnemonic,
    Register,
    Symbol,
}

impl IdentifierType {
    fn mnemonics<'a>() -> [&'a str; Instruction::VARIANT_COUNT] {
        Instruction::mnemonics()
    }

    fn registers<'a>() -> [&'a str; Register::VARIANT_COUNT] {
        Register::variants()
    }
}

impl From<&[u8]> for IdentifierType {
    fn from(value: &[u8]) -> Self {
        if let Some(_) = Self::mnemonics().iter().find(|v| v.as_bytes() == value) {
            return Self::Mnemonic;
        };

        if let Some(_) = Self::registers().iter().find(|v| v.as_bytes() == value) {
            return Self::Register;
        };

        Self::Symbol
    }
}

fn extract_ident(lex: &mut logos::Lexer<Token>) -> Result<IdentifierType, LexingError> {
    let value = lex.slice();
    if !value.is_ascii() {
        return Err(LexingError::NonAsciiCharacter(lex.span().end));
    }

    Ok(value.into())
}

#[derive(Logos, Debug, PartialEq, EnumCount)]
#[logos(source = [u8])]
#[logos(skip r"[ \t\f]+")] // Ignore this regex pattern between tokens
#[logos(extras = State)]
#[logos(error = LexingError)]
pub enum Token {
    #[regex(r#"\s\d+"#)]
    Decimal,

    #[regex(r#"\w+:[\s*|\n]"#)] //suffix `:` followed by any whitespace or enter(EOL)
    LabelDef,
    #[regex(r#"\.\w+"#)]
    Directive,
    #[regex(r#"\"(\\.|[^\\"])*\""#)] // https://www.lysator.liu.se/c/ANSI-C-grammar-l.html
    LiteralString,
    #[regex(r#"\s0x[0-9a-fA-F]+[\s(]"#)]
    LiteralHex,
    #[regex(r#"\s0b[01]+(\s*|\()"#)]
    LiteralBinary,
    #[regex(r#"\w+"#, extract_ident)]
    Identifier(IdentifierType),

    #[token(b")")]
    ParenR,
    #[token(b"-")]
    Negative,
    #[token(b"+")]
    Positive,
    #[token(b"\'")]
    QuoteSingle,
    #[token(b".")]
    Dot,
    #[token(b",")]
    Comma,
    #[token(b"(")]
    ParenL,
    #[token(b"\"")]
    QuoteDouble,

    #[token(b":")]
    Colon,
    #[token(b"\n", on_newline, priority = 2)]
    Eol,

    #[regex(r"#[^\n]*", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    CommentSingleLine,
    // #[token(b"/*", |lex| { lex.extras.in_block_comments = true; logos::Skip }, priority = 2)]
    // CommentBlockStart,
    // #[token(b"*/", |lex| { lex.extras.in_block_comments = false; logos::Skip }, priority = 3)]
    // CommentBlockEnd,
}

pub struct Tokens([Token; Token::VARIANT_COUNT - 1]); // 1 here is the

pub struct Lexer;

impl Lexer {
    fn new() -> Lexer {
        // let regex_set = RegexSet::new(patterns).unwrap();
        // let regexes = patterns.iter().map(|p| Regex::new(p).unwrap()).collect();
        // Lexer { regex_set, regexes }
        Lexer
    }

    pub fn tokenize<'a>(&self, input: &'a [u8]) -> Result<(), LexingError> {
        // TODO: get the token and the span, ditch others
        let mut lex = Token::lexer(input);
        while let Some(sequence) = lex.next() {
            // match sequence {
            //     Ok(t) => todo!(),
            //     Err(_) => todo!(),
            // }
            let token = sequence?;
            let span = lex.span();
            // todo:

            let word = String::from_utf8(lex.slice().to_vec()).unwrap();
            println!("Token Match: {:?} {}", token, word);
            // match token {
            //      =>
            //     _ =>
            // }
        }

        Ok(())

        // while pos < len {
        //     let target = &input[pos..];
        //     // Find the first matching pattern
        //     let matches = self.regex_set.matches(target);
        //     println!("Total match: {:?}", matches);
        //     let Some(pattern_id) = matches.iter().next() else {
        //         return Err(TokenizeError::UnclosedComment(pos));
        //     };

        //     let re = &self.regexes[pattern_id];
        //     let Some(m) = re.find(target) else {
        //         continue;
        //     };
        //     let start = pos + m.start();
        //     let end = pos + m.end();

        //     println!("Pattern ID: {pattern_id}");
        //     println!("");

        //     match pattern_id {
        //         0 => {
        //             // continue;
        //             let reg = m.as_bytes();
        //             let memo = String::from_utf8(reg.to_owned());
        //             let span = (start, end);
        //             println!("Block comments Bytes: {:?}, pos: {:?} ", memo, span);
        //             // tokens.push(Token {
        //             //     typ: TokenType::BlockComment,
        //             //     span: (start, end),
        //             // });
        //         }
        //         1 => {
        //             // continue;
        //             let reg = m.as_bytes();
        //             let memo = String::from_utf8(reg.to_owned());
        //             let span = (start, end);
        //             println!("Block comments Bytes: {:?}, pos: {:?} ", memo, span);
        //             // tokens.push(Token {
        //             //     typ: TokenType::Comment,
        //             //     span: (start, end),
        //             // });
        //         }
        //         2 => {
        //             // Mnemonic
        //             let mnemonic = m.as_bytes();
        //             let memo = String::from_utf8(mnemonic.to_owned());
        //             let span = (start, end);
        //             println!("Mnemonic Bytes: {:?}, pos: {:?} ", memo, span);
        //             tokens.push(Token {
        //                 typ: TokenType::Mnemonic(mnemonic),
        //                 span: (start, end),
        //             });
        //         }
        //         3 => {
        //             // Directive
        //             let directive = m.as_bytes();
        //             tokens.push(Token {
        //                 typ: TokenType::Directive(directive),
        //                 span: (start, end),
        //             });
        //         }
        //         4 => {
        //             // Register
        //             let reg = m.as_bytes();
        //             let memo = String::from_utf8(reg.to_owned());
        //             let span = (start, end);
        //             println!("Register Bytes: {:?}, pos: {:?} ", memo, span);
        //             tokens.push(Token {
        //                 typ: TokenType::Register(reg),
        //                 span: (start, end),
        //             });
        //         }
        //         5 => {
        //             // Label definition

        //             // let label = m.as_str().trim_end_matches(':').to_string();
        //             let label = m.as_bytes().trim_ascii_end();
        //             tokens.push(Token {
        //                 typ: TokenType::LabelDef(label),
        //                 span: (start, end),
        //             });
        //         }
        //         6 => {
        //             // Immediate
        //             let b = m.as_bytes();
        //             let memo = String::from_utf8(b.to_owned());
        //             let span = (start, end);
        //             println!("Immediate Bytes: {:?}, pos: {:?} ", memo, span);
        //             // println!("Imm Bytes: {:?}", memo);

        //             // let imm = i32::from_ne_bytes([b[0], b[1], b[2], b[3]]);
        //             let imm = 100;
        //             tokens.push(Token {
        //                 typ: TokenType::Immediate(imm),
        //                 span: (start, end),
        //             });
        //         }
        //         7 => {
        //             // Punctuation
        //             let punct = m.as_bytes();
        //             let typ = match punct {
        //                 b"," => TokenType::Comma,
        //                 b"(" => TokenType::LParen,
        //                 b")" => TokenType::RParen,
        //                 _ => unreachable!(),
        //             };
        //             tokens.push(Token {
        //                 typ,
        //                 span: (start, end),
        //             });
        //         }
        //         8 => {
        //             // Immediate
        //             let b = m.as_bytes();
        //             let memo = String::from_utf8(b.to_owned());
        //             let span = (start, end);
        //             println!("Whitespace Bytes: {:?}, pos: {:?} ", memo, span);
        //             // println!("Imm Bytes: {:?}", memo);

        //             // let imm = i32::from_ne_bytes([b[0], b[1], b[2], b[3]]);
        //             tokens.push(Token {
        //                 typ: TokenType::Whitespace,
        //                 span: (start, end),
        //             });
        //         }
        //         _ => unreachable!(),
        //     }

        //     pos = end;
        // }
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Read};

    use super::*;

    #[test]
    fn t_tokenize() {
        let lex = Lexer::new();
        match File::open("test.asm") {
            Ok(mut file) => {
                let mut buffer = Vec::new();

                // read the whole file
                file.read_to_end(&mut buffer).unwrap();
                println!("Buffer : {:?}", &buffer[0..6]);
                println!("");

                let token = lex.tokenize(&buffer);
            }
            Err(e) => println!("File Error: {:?}", e),
        }
    }
}
