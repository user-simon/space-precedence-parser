/// Lexical token that's used for parsing. Contains the value of the token as well as its spacing from the
/// preceeding token
#[derive(Clone, Copy, Debug)]
pub enum Token<'a> {
    Number(f64, usize), 
    Symbol(char, usize), 
    Word(&'a str, usize), 
}

impl Token<'_> {
    pub fn spacing(&self) -> usize {
        match self {
            Token::Number(_, s) => *s,
            Token::Symbol(_, s) => *s,
            Token::Word(_, s)   => *s,
        }
    }
}

/// Token iterator from an input string
pub struct Tokens<'a> {
    /// String being tokenized
    pub string: &'a str, 
    /// Cached value of the next token, set by `Tokens::peek`. Allows for reading a token without consuming
    /// it
    peek: Option<Token<'a>>, 
}

impl<'a> Tokens<'a> {
    /// Reads the next token and stores it in the peek cache, such that it can still be the next token
    /// yielded by `<Tokens as Iterator>::next`
    pub fn peek(&mut self) -> Option<&Token<'a>> {
        self.peek = self.peek.or_else(|| self.next());
        self.peek.as_ref()
    }
}

impl<'a> From<&'a str> for Tokens<'a> {
    fn from(string: &'a str) -> Self {
        Tokens {
            string, 
            peek: None, 
        }
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Token<'a>;

    /// Removes lexemes from the front of string in chunks of one token each
    fn next(&mut self) -> Option<Self::Item> {
        // if a token has been peeked, consume and return it. otherwise, tokenize input as normal
        if let Some(peek) = self.peek.take() {
            return Some(peek)
        }

        // removes all leading spaces, later storing the length of it inside the token
        let spacing = gobble(Category::Whitespace, &mut self.string);
        let spacing = spacing.chars().count();

        // read the first character in the input and produce a token based on what type it is
        let first = self.string.chars().nth(0)?;
        let token = match Category::from(first) {
            Category::Letter => {
                let lexeme = gobble(Category::Letter, &mut self.string);
                Token::Word(lexeme, spacing)
            }
            Category::Digit => {
                let lexeme = gobble(Category::Digit, &mut self.string);
                let number = lexeme.parse().expect("Invalid floating-point number");
                Token::Number(number, spacing)
            }
            Category::Symbol => {
                self.string = &self.string[1..];
                Token::Symbol(first, spacing)
            }
            Category::Whitespace => unreachable!("All leading spaces are removed by `gobble`"), 
        };
        Some(token)
    }
}

/// Utility to store the type of a character
#[derive(PartialEq)]
enum Category {
    Letter, 
    Digit, 
    Symbol, 
    Whitespace, 
}

impl From<char> for Category {
    fn from(c: char) -> Self {
        if c.is_alphabetic() || c == '_' {
            Category::Letter
        } else if c.is_ascii_digit() || c == '.' {
            Category::Digit
        } else if c.is_whitespace() {
            Category::Whitespace
        } else {
            Category::Symbol
        }
    }
}

/// Utility that consumes as many symbols of the given `Category` as possible from the front of the string
fn gobble<'a>(category: Category, string: &mut &'a str) -> &'a str {
    let (lexeme, rest) = string
        .find(|c| Category::from(c) != category)
        .map(|index| string.split_at(index))
        .unwrap_or((string, ""));
    *string = rest;
    lexeme
}
