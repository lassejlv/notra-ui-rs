//! Notra's code block: a dualtone panel with a filename or tabs on the shell, a
//! syntax-highlighted body and a copy button, plus the one-line command variant.
//!
//! The highlighter follows sugar-high (the site's highlighter): the same token
//! classes, coloured by [`crate::theme::Syntax`].

use std::ops::Range;

use crate::theme::{Color, Syntax};

/// Which grammar to highlight with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Language {
    /// JavaScript with JSX (sugar-high's default).
    #[default]
    JavaScript,
    /// TypeScript and TSX: adds the TS keywords.
    TypeScript,
    Json,
    Bash,
}

/// sugar-high's token classes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Identifier,
    Keyword,
    String,
    /// Capitalised names, numbers and `null`.
    Class,
    Property,
    /// JSX tag names.
    Entity,
    /// Text between JSX tags.
    JsxLiterals,
    Sign,
    Comment,
    Space,
}

impl TokenKind {
    /// The token's colour (`--sh-*`); spaces keep the text colour.
    pub fn color(self, syntax: &Syntax) -> Option<Color> {
        Some(match self {
            TokenKind::Identifier => syntax.identifier,
            TokenKind::Keyword => syntax.keyword,
            TokenKind::String => syntax.string,
            TokenKind::Class => syntax.class,
            TokenKind::Property => syntax.property,
            TokenKind::Entity => syntax.entity,
            TokenKind::JsxLiterals => syntax.jsx_literals,
            TokenKind::Sign => syntax.sign,
            TokenKind::Comment => syntax.comment,
            TokenKind::Space => return None,
        })
    }
}

/// One coloured run: its class and byte range in the source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub range: Range<usize>,
}

const JS_KEYWORDS: &[&str] = &[
    "for",
    "do",
    "while",
    "if",
    "else",
    "return",
    "function",
    "var",
    "let",
    "const",
    "true",
    "false",
    "undefined",
    "this",
    "new",
    "delete",
    "typeof",
    "in",
    "instanceof",
    "void",
    "break",
    "continue",
    "switch",
    "case",
    "default",
    "throw",
    "try",
    "catch",
    "finally",
    "debugger",
    "with",
    "yield",
    "async",
    "await",
    "class",
    "extends",
    "super",
    "import",
    "export",
    "from",
    "static",
];
const TS_KEYWORDS: &[&str] = &[
    "type",
    "interface",
    "enum",
    "implements",
    "readonly",
    "abstract",
    "declare",
    "namespace",
    "module",
    "private",
    "protected",
    "public",
    "override",
    "keyof",
    "infer",
    "is",
    "asserts",
    "satisfies",
    "as",
    "unknown",
    "never",
    "any",
    "number",
    "string",
    "boolean",
    "bigint",
    "symbol",
    "object",
];
const JSON_KEYWORDS: &[&str] = &["true", "false", "null"];
const BASH_KEYWORDS: &[&str] = &[
    "case", "coproc", "do", "done", "elif", "else", "esac", "export", "fi", "for", "function",
    "if", "in", "local", "readonly", "return", "select", "then", "time", "until", "while",
];
const SIGNS: &str = "+-*/%=!&|^~?:.,;()[]{}<>#@\\";

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

fn is_class(word: &str) -> bool {
    word == "null"
        || word
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit() || c.is_uppercase())
}

/// Splits `code` into lines of coloured tokens. Byte ranges index `code`; line
/// breaks are not tokens.
pub fn highlight(code: &str, language: Language) -> Vec<Vec<Token>> {
    let tokens = match language {
        Language::JavaScript => Lexer::new(code, JS_KEYWORDS, &[]).script(),
        Language::TypeScript => Lexer::new(code, JS_KEYWORDS, TS_KEYWORDS).script(),
        Language::Json => Lexer::new(code, JSON_KEYWORDS, &[]).plain(Comments::CLike, true),
        Language::Bash => Lexer::new(code, BASH_KEYWORDS, &[]).plain(Comments::Hash, false),
    };
    split_lines(code, tokens)
}

fn split_lines(code: &str, tokens: Vec<Token>) -> Vec<Vec<Token>> {
    let mut lines = vec![Vec::new()];
    for token in tokens {
        let mut start = token.range.start;
        for (offset, _) in code[token.range.clone()].match_indices('\n') {
            let end = token.range.start + offset;
            let line_end = if code[..end].ends_with('\r') && end > start {
                end - 1
            } else {
                end
            };
            if line_end > start
                && let Some(line) = lines.last_mut()
            {
                line.push(Token {
                    kind: token.kind,
                    range: start..line_end,
                });
            }
            lines.push(Vec::new());
            start = end + 1;
        }
        if token.range.end > start
            && let Some(line) = lines.last_mut()
        {
            line.push(Token {
                kind: token.kind,
                range: start..token.range.end,
            });
        }
    }
    lines
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Comments {
    CLike,
    Hash,
}

/// JSX nesting: inside an opening or closing tag, or among an element's children.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Jsx {
    OpenTag,
    CloseTag,
    Children,
}

struct Lexer<'a> {
    code: &'a str,
    keywords: &'a [&'a str],
    extra_keywords: &'a [&'a str],
    pos: usize,
    tokens: Vec<Token>,
    /// The last significant token's text and kind.
    last: Option<(Range<usize>, TokenKind)>,
    before_last: Option<(Range<usize>, TokenKind)>,
    jsx: Vec<Jsx>,
    /// Open `{` inside the innermost JSX expression or template substitution.
    braces: Vec<u32>,
}

impl<'a> Lexer<'a> {
    fn new(code: &'a str, keywords: &'a [&'a str], extra_keywords: &'a [&'a str]) -> Self {
        Self {
            code,
            keywords,
            extra_keywords,
            pos: 0,
            tokens: Vec::new(),
            last: None,
            before_last: None,
            jsx: Vec::new(),
            braces: Vec::new(),
        }
    }

    fn peek(&self, ahead: usize) -> Option<char> {
        self.code[self.pos..].chars().nth(ahead)
    }

    fn push(&mut self, kind: TokenKind, range: Range<usize>) {
        if range.is_empty() {
            return;
        }
        let blank = self.code[range.clone()].chars().all(char::is_whitespace);
        if kind != TokenKind::Space && !blank {
            self.before_last = self.last.take();
            self.last = Some((range.clone(), kind));
        }
        match self.tokens.last_mut() {
            Some(prev) if prev.kind == kind && prev.range.end == range.start => {
                prev.range.end = range.end;
            }
            _ => self.tokens.push(Token { kind, range }),
        }
    }

    fn last_text(&self) -> &str {
        self.last
            .as_ref()
            .map_or("", |(r, _)| &self.code[r.clone()])
    }

    fn is_keyword(&self, word: &str) -> bool {
        self.keywords.contains(&word) || self.extra_keywords.contains(&word)
    }

    fn take_while(&mut self, f: impl Fn(char) -> bool) -> Range<usize> {
        let start = self.pos;
        while let Some(c) = self.peek(0) {
            if !f(c) {
                break;
            }
            self.pos += c.len_utf8();
        }
        start..self.pos
    }

    fn quoted(&mut self, quote: char) -> Range<usize> {
        let start = self.pos;
        self.pos += quote.len_utf8();
        while let Some(c) = self.peek(0) {
            self.pos += c.len_utf8();
            if c == '\\' {
                if let Some(next) = self.peek(0) {
                    self.pos += next.len_utf8();
                }
            } else if c == quote || (c == '\n' && quote != '`') {
                break;
            }
        }
        start..self.pos
    }

    fn comment(&mut self, block: bool) -> Range<usize> {
        let start = self.pos;
        if block {
            match self.code[self.pos + 2..].find("*/") {
                Some(end) => self.pos += 2 + end + 2,
                None => self.pos = self.code.len(),
            }
        } else {
            match self.code[self.pos..].find('\n') {
                Some(end) => self.pos += end,
                None => self.pos = self.code.len(),
            }
        }
        start..self.pos
    }

    /// sugar-high's general lexer (JSON, shell).
    fn plain(mut self, comments: Comments, keys: bool) -> Vec<Token> {
        while let Some(c) = self.peek(0) {
            let next = self.peek(1);
            if comments == Comments::Hash && c == '#'
                || comments == Comments::CLike && c == '/' && matches!(next, Some('/' | '*'))
            {
                let block = comments == Comments::CLike && next == Some('*');
                let range = self.comment(block);
                self.push(TokenKind::Comment, range);
            } else if c.is_whitespace() {
                let range = self.take_while(char::is_whitespace);
                self.push(TokenKind::Space, range);
            } else if c == '"' || c == '\'' {
                let range = self.quoted(c);
                let kind = if keys && self.followed_by(':') {
                    TokenKind::Property
                } else {
                    TokenKind::String
                };
                self.push(kind, range);
            } else if is_word(c) {
                let range = self.word();
                let word = &self.code[range.clone()];
                let kind = if self.is_keyword(word) {
                    TokenKind::Keyword
                } else if self.last_text() == "." || keys && self.followed_by(':') {
                    TokenKind::Property
                } else if is_class(word) {
                    TokenKind::Class
                } else {
                    TokenKind::Identifier
                };
                self.push(kind, range);
            } else {
                let start = self.pos;
                self.pos += c.len_utf8();
                let kind = if SIGNS.contains(c) {
                    TokenKind::Sign
                } else {
                    TokenKind::String
                };
                self.push(kind, start..self.pos);
            }
        }
        self.tokens
    }

    fn word(&mut self) -> Range<usize> {
        let start = self.pos;
        let range = self.take_while(is_word);
        let leading_digit = self.code[start..].starts_with(|c: char| c.is_ascii_digit());
        if leading_digit
            && self.peek(0) == Some('.')
            && self.peek(1).is_some_and(|c| c.is_ascii_digit())
        {
            self.pos += 1;
            self.take_while(is_word);
            return start..self.pos;
        }
        range
    }

    fn followed_by(&self, sign: char) -> bool {
        self.code[self.pos..]
            .trim_start_matches([' ', '\t'])
            .starts_with(sign)
    }

    /// Whether a `<` here opens a JSX tag rather than comparing.
    fn opens_jsx(&self) -> bool {
        let follows_value = matches!(
            self.last.as_ref().map(|(_, k)| *k),
            Some(TokenKind::Identifier | TokenKind::Class | TokenKind::Property)
        ) || matches!(self.last_text(), ")" | "]");
        !follows_value
            && self
                .peek(1)
                .is_some_and(|c| c.is_alphabetic() || c == '>' || c == '/')
    }

    /// The JavaScript runtime lexer with JSX and template strings.
    fn script(mut self) -> Vec<Token> {
        while let Some(c) = self.peek(0) {
            let next = self.peek(1);
            match self.jsx.last().copied() {
                Some(Jsx::Children) if self.braces.last() != Some(&0) || self.braces.is_empty() => {
                    if c == '<' {
                        self.jsx_tag_start();
                    } else if c == '{' {
                        self.pos += 1;
                        self.push(TokenKind::Sign, self.pos - 1..self.pos);
                        self.braces.push(0);
                    } else {
                        let range = self.take_while(|c| c != '<' && c != '{');
                        self.push(TokenKind::JsxLiterals, range);
                    }
                    continue;
                }
                Some(Jsx::OpenTag | Jsx::CloseTag) if self.braces.last() != Some(&0) => {
                    self.jsx_in_tag(c, next);
                    continue;
                }
                _ => {}
            }
            if c == '/' && matches!(next, Some('/' | '*')) {
                let range = self.comment(next == Some('*'));
                self.push(TokenKind::Comment, range);
            } else if c.is_whitespace() {
                let range = self.take_while(char::is_whitespace);
                self.push(TokenKind::Space, range);
            } else if c == '"' || c == '\'' {
                let range = self.quoted(c);
                self.push(TokenKind::String, range);
            } else if c == '`' {
                self.template();
            } else if c == '<' && self.opens_jsx() {
                self.jsx_tag_start();
            } else if c == '{' || c == '}' {
                self.pos += 1;
                self.push(TokenKind::Sign, self.pos - 1..self.pos);
                if let Some(depth) = self.braces.last_mut() {
                    if c == '{' {
                        *depth += 1;
                    } else if *depth == 0 {
                        self.braces.pop();
                    } else {
                        *depth -= 1;
                    }
                }
            } else if is_word(c) {
                let range = self.word();
                let kind = self.classify_word(&self.code[range.clone()]);
                self.push(kind, range);
            } else {
                let start = self.pos;
                self.pos += c.len_utf8();
                let kind = if SIGNS.contains(c) {
                    TokenKind::Sign
                } else {
                    TokenKind::String
                };
                self.push(kind, start..self.pos);
            }
        }
        self.tokens
    }

    fn classify_word(&self, word: &str) -> TokenKind {
        let after_dot = self.last_text() == ".";
        if self.is_keyword(word) {
            return if after_dot {
                TokenKind::Identifier
            } else {
                TokenKind::Keyword
            };
        }
        if is_class(word) {
            return TokenKind::Class;
        }
        let property = after_dot
            && self
                .before_last
                .as_ref()
                .is_some_and(|(r, _)| self.code[r.clone()].chars().all(is_word));
        if property {
            TokenKind::Property
        } else {
            TokenKind::Identifier
        }
    }

    fn template(&mut self) {
        let start = self.pos;
        self.pos += 1;
        loop {
            match self.peek(0) {
                None => {
                    self.push(TokenKind::String, start..self.pos);
                    return;
                }
                Some('\\') => {
                    self.pos += 1;
                    if let Some(c) = self.peek(0) {
                        self.pos += c.len_utf8();
                    }
                }
                Some('`') => {
                    self.pos += 1;
                    self.push(TokenKind::String, start..self.pos);
                    return;
                }
                Some('$') if self.peek(1) == Some('{') => {
                    self.push(TokenKind::String, start..self.pos);
                    self.pos += 2;
                    self.push(TokenKind::Sign, self.pos - 2..self.pos);
                    let close = self.substitution();
                    if !close {
                        return;
                    }
                    return self.template_rest();
                }
                Some(c) => self.pos += c.len_utf8(),
            }
        }
    }

    /// Lexes a template substitution up to its closing brace; false at end of input.
    fn substitution(&mut self) -> bool {
        let mut depth = 0u32;
        while let Some(c) = self.peek(0) {
            if c == '}' && depth == 0 {
                self.pos += 1;
                self.push(TokenKind::Sign, self.pos - 1..self.pos);
                return true;
            }
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
            }
            if c.is_whitespace() {
                let range = self.take_while(char::is_whitespace);
                self.push(TokenKind::Space, range);
            } else if c == '"' || c == '\'' {
                let range = self.quoted(c);
                self.push(TokenKind::String, range);
            } else if is_word(c) {
                let range = self.word();
                let kind = self.classify_word(&self.code[range.clone()]);
                self.push(kind, range);
            } else {
                self.pos += c.len_utf8();
                self.push(TokenKind::Sign, self.pos - c.len_utf8()..self.pos);
            }
        }
        false
    }

    /// The rest of a template literal after a substitution.
    fn template_rest(&mut self) {
        let start = self.pos;
        while let Some(c) = self.peek(0) {
            match c {
                '\\' => {
                    self.pos += 1;
                    if let Some(c) = self.peek(0) {
                        self.pos += c.len_utf8();
                    }
                }
                '`' => {
                    self.pos += 1;
                    self.push(TokenKind::String, start..self.pos);
                    return;
                }
                '$' if self.peek(1) == Some('{') => {
                    self.push(TokenKind::String, start..self.pos);
                    self.pos += 2;
                    self.push(TokenKind::Sign, self.pos - 2..self.pos);
                    if self.substitution() {
                        self.template_rest();
                    }
                    return;
                }
                c => self.pos += c.len_utf8(),
            }
        }
        self.push(TokenKind::String, start..self.pos);
    }

    fn jsx_tag_start(&mut self) {
        let closing = self.peek(1) == Some('/');
        let len = if closing { 2 } else { 1 };
        self.pos += len;
        self.push(TokenKind::Sign, self.pos - len..self.pos);
        self.jsx
            .push(if closing { Jsx::CloseTag } else { Jsx::OpenTag });
        if self.peek(0) == Some('>') {
            self.pos += 1;
            self.push(TokenKind::Sign, self.pos - 1..self.pos);
            self.close_tag(closing);
        }
    }

    /// After a tag's `>`: an opening tag starts its children, a closing tag ends them.
    fn close_tag(&mut self, closing: bool) {
        self.jsx.pop();
        if closing {
            if self.jsx.last() == Some(&Jsx::Children) {
                self.jsx.pop();
            }
        } else {
            self.jsx.push(Jsx::Children);
        }
    }

    fn jsx_in_tag(&mut self, c: char, next: Option<char>) {
        let closing = self.jsx.last() == Some(&Jsx::CloseTag);
        if c.is_whitespace() {
            let range = self.take_while(char::is_whitespace);
            self.push(TokenKind::Space, range);
        } else if c == '/' && next == Some('>') {
            self.pos += 2;
            self.push(TokenKind::Sign, self.pos - 2..self.pos);
            self.jsx.pop();
        } else if c == '>' {
            self.pos += 1;
            self.push(TokenKind::Sign, self.pos - 1..self.pos);
            self.close_tag(closing);
        } else if c == '"' || c == '\'' {
            let range = self.quoted(c);
            self.push(TokenKind::String, range);
        } else if c == '{' {
            self.pos += 1;
            self.push(TokenKind::Sign, self.pos - 1..self.pos);
            self.braces.push(0);
        } else if is_word(c) || c == '-' {
            let range = self.take_while(|c| is_word(c) || c == '-' || c == '.');
            let kind = if matches!(self.last_text(), "<" | "</") {
                TokenKind::Entity
            } else {
                TokenKind::Property
            };
            self.push(kind, range);
        } else {
            self.pos += c.len_utf8();
            self.push(TokenKind::Sign, self.pos - c.len_utf8()..self.pos);
        }
    }
}

/// Box metrics, in logical pixels.
pub mod metrics {
    pub const SHELL_RADIUS: f32 = 16.0;
    /// `p-0.5 pt-0`.
    pub const SHELL_PADDING: f32 = 2.0;
    pub const HEADER_HEIGHT: f32 = 36.0;
    pub const HEADER_GAP: f32 = 12.0;
    pub const HEADER_PADDING_START: f32 = 10.0;
    pub const HEADER_PADDING_END: f32 = 4.0;
    /// The filename keeps `pe-2` off the tabs' end.
    pub const FILENAME_PADDING_END: f32 = 8.0;
    pub const BODY_RADIUS: f32 = 14.0;
    pub const CODE_PADDING: f32 = 12.0;
    /// `pe-10`: room for the copy button.
    pub const CODE_PADDING_END: f32 = 40.0;
    pub const CODE_SIZE: f32 = 12.0;
    /// `leading-relaxed`.
    pub const CODE_LINE_HEIGHT: f32 = 19.5;
    /// The copy button floats `end-1 top-1`.
    pub const COPY_INSET: f32 = 4.0;
    pub const COPY_RADIUS: f32 = 8.0;
    /// The command variant: `h-9 rounded-lg ps-3 pe-1`.
    pub const COMMAND_HEIGHT: f32 = 36.0;
    pub const COMMAND_RADIUS: f32 = 8.0;
    pub const COMMAND_PADDING_START: f32 = 12.0;
    pub const COMMAND_PADDING_END: f32 = 4.0;
}

/// The copy button fades in on hover: `transition-opacity duration-150`.
pub const COPY_REVEAL: crate::motion::Tween = crate::motion::Tween::new(
    std::time::Duration::from_millis(150),
    crate::motion::ease::IN_OUT,
);

/// Default accessible names.
pub mod labels {
    pub const COMMAND: &str = "Command";
    pub const SNIPPET: &str = "Snippet";
}

/// The code block demo: a framework switcher over two snippets, then a command.
pub mod demo {
    pub const FRAMEWORKS: [(&str, &str, &str); 2] = [
        ("Next.js", "proxy.ts", NEXT),
        ("Astro", "src/middleware.ts", ASTRO),
    ];
    pub const NEXT: &str = "import { withGeo } from \"@usenotra/geo/next\";\n\nexport const proxy = withGeo({\n  token: process.env.NOTRA_GEO_TOKEN,\n});\n\nexport const config = { matcher: [\"/((?!_next|api).*)\"] };";
    pub const ASTRO: &str = "import { geoMiddleware } from \"@usenotra/geo/astro\";\n\nexport const onRequest = geoMiddleware({\n  token: import.meta.env.NOTRA_GEO_TOKEN,\n});";
    pub const COMMAND: &str = "NOTRA_GEO_TOKEN=ntr_geo_••••••••";
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(code: &str, language: Language) -> Vec<(TokenKind, String)> {
        highlight(code, language)
            .into_iter()
            .flatten()
            .filter(|t| t.kind != TokenKind::Space)
            .map(|t| (t.kind, code[t.range].to_owned()))
            .collect()
    }

    #[test]
    fn matches_sugar_high_on_the_demo() {
        let tokens = kinds(
            "  token: process.env.NOTRA_GEO_TOKEN,",
            Language::JavaScript,
        );
        use TokenKind::*;
        assert_eq!(
            tokens,
            vec![
                (Identifier, "token".into()),
                (Sign, ":".into()),
                (Identifier, "process".into()),
                (Sign, ".".into()),
                (Property, "env".into()),
                (Sign, ".".into()),
                (Class, "NOTRA_GEO_TOKEN".into()),
                (Sign, ",".into()),
            ]
        );
        let tokens = kinds(
            "import { withGeo } from \"@usenotra/geo/next\";",
            Language::JavaScript,
        );
        assert_eq!(tokens[0], (Keyword, "import".into()));
        assert_eq!(tokens[4], (Keyword, "from".into()));
        assert_eq!(tokens[5], (String, "\"@usenotra/geo/next\"".into()));
    }

    #[test]
    fn lines_split_on_breaks() {
        let lines = highlight(demo::NEXT, Language::JavaScript);
        assert_eq!(lines.len(), demo::NEXT.lines().count());
        assert!(lines[1].is_empty());
    }

    #[test]
    fn jsx_tags_attributes_and_text() {
        use TokenKind::*;
        let tokens = kinds(
            "const a = <Card size=\"sm\">Hi {name}</Card>;",
            Language::JavaScript,
        );
        assert!(tokens.contains(&(Entity, "Card".into())));
        assert!(tokens.contains(&(Property, "size".into())));
        assert!(tokens.contains(&(JsxLiterals, "Hi ".into())));
        assert!(tokens.contains(&(Identifier, "name".into())));
        assert_eq!(tokens.last(), Some(&(Sign, ">;".into())));
    }

    #[test]
    fn templates_comments_json_and_bash() {
        use TokenKind::*;
        let tokens = kinds("const s = `a ${b} c`; // done", Language::TypeScript);
        assert!(tokens.contains(&(Identifier, "b".into())));
        assert_eq!(tokens.last(), Some(&(Comment, "// done".into())));
        let json = kinds(
            "{ \"name\": \"notra\", \"n\": 2, \"ok\": true }",
            Language::Json,
        );
        assert!(json.contains(&(Property, "\"name\"".into())));
        assert!(json.contains(&(String, "\"notra\"".into())));
        assert!(json.contains(&(Class, "2".into())));
        assert!(json.contains(&(Keyword, "true".into())));
        let bash = kinds("export PATH=$HOME/bin # add", Language::Bash);
        assert_eq!(bash[0], (Keyword, "export".into()));
        assert_eq!(bash.last(), Some(&(Comment, "# add".into())));
    }

    #[test]
    fn every_kind_but_space_has_a_colour() {
        let s = crate::Theme::light().syntax;
        assert!(TokenKind::Keyword.color(&s).is_some());
        assert!(TokenKind::Space.color(&s).is_none());
    }
}
