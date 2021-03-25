pub mod token {
    use std::collections::HashMap;
    use std::fmt;

    #[derive(Debug, Eq, PartialEq)]
    pub enum Token<'a> {
        Lit(&'a str),
        Var(&'a str),
    }

    impl<'a> Token<'a> {
        pub fn as_var(&self) -> Option<&'a str> {
            if let Token::Var(s) = self {
                Some(s)
            } else {
                None
            }
        }

        pub fn eval(&self, ctx: &'a HashMap<String, String>) -> &'a str {
            match self {
                Token::Lit(s) => s,
                Token::Var(n) => ctx[*n].as_str(),
            }
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum Error {
        ExpectedDoubleRightBraces(usize),
        UnexpectedEndOfFile,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::ExpectedDoubleRightBraces(pos) => {
                    write!(f, "expected \"}}\" at position {}", pos)
                }
                Error::UnexpectedEndOfFile => {
                    write!(f, "unexpected end of file")
                }
            }
        }
    }

    impl std::error::Error for Error {}

    enum State {
        Lit,
        Var,
    }

    pub struct Tokenizer<'a> {
        state: State,
        corpus: &'a str,
        offset: usize,
        start: usize,
    }

    impl<'a> Tokenizer<'a> {
        pub fn new(corpus: &'a str) -> Self {
            let (state, offset) = if corpus.starts_with("{{") {
                (State::Var, 2)
            } else {
                (State::Lit, 0)
            };
            Tokenizer {
                state,
                corpus,
                offset,
                start: 0,
            }
        }
    }

    impl<'a> Iterator for Tokenizer<'a> {
        type Item = Result<Token<'a>, Error>;
        fn next(&mut self) -> Option<Result<Token<'a>, Error>> {
            if self.corpus.is_empty() {
                None
            } else {
                let (token, state, delta, offset) = match self.state {
                    State::Lit => {
                        if let Some(i) = self.corpus[self.offset..].find("{{") {
                            (
                                Ok(Token::Lit(&self.corpus[..self.offset + i])),
                                State::Var,
                                self.offset + i,
                                0,
                            )
                        } else {
                            (
                                Ok(Token::Lit(&self.corpus)),
                                State::Lit,
                                self.corpus.len(),
                                0,
                            )
                        }
                    }
                    State::Var => {
                        if let Some(i) = self.corpus[2..].find(|c| c == '{' || c == '}') {
                            if self.corpus[2 + i..].starts_with("}}") {
                                let state = if self.corpus[4 + i..].starts_with("{{") {
                                    State::Var
                                } else {
                                    State::Lit
                                };
                                (
                                    Ok(Token::Var(&self.corpus[2..2 + i].trim())),
                                    state,
                                    4 + i,
                                    0,
                                )
                            } else {
                                (
                                    Err(Error::ExpectedDoubleRightBraces(self.start + 2 + i)),
                                    State::Lit,
                                    0,
                                    2 + i,
                                )
                            }
                        } else {
                            (Err(Error::UnexpectedEndOfFile), State::Lit, 0, 2)
                        }
                    }
                };
                self.state = state;
                self.corpus = &self.corpus[delta..];
                self.offset = offset;
                self.start += delta;
                Some(token)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn empty() {
            let mut t = Tokenizer::new("");
            assert_eq!(t.next(), None);
        }

        #[test]
        fn lit() {
            let mut t = Tokenizer::new("lorem");
            assert_eq!(t.next(), Some(Ok(Token::Lit("lorem"))));
            assert_eq!(t.next(), None);
        }

        #[test]
        fn var() {
            let mut t = Tokenizer::new("{{lorem}}");
            assert_eq!(t.next(), Some(Ok(Token::Var("lorem"))));
            assert_eq!(t.next(), None);
        }

        #[test]
        fn var_trim() {
            let mut t = Tokenizer::new("{{ lorem }}");
            assert_eq!(t.next(), Some(Ok(Token::Var("lorem"))));
            assert_eq!(t.next(), None);
        }

        #[test]
        fn var_var() {
            let mut t = Tokenizer::new("{{lorem}}{{ipsum}}");
            assert_eq!(t.next(), Some(Ok(Token::Var("lorem"))));
            assert_eq!(t.next(), Some(Ok(Token::Var("ipsum"))));
            assert_eq!(t.next(), None);
        }

        #[test]
        fn lit_var() {
            let mut t = Tokenizer::new("lorem{{ipsum}}");
            assert_eq!(t.next(), Some(Ok(Token::Lit("lorem"))));
            assert_eq!(t.next(), Some(Ok(Token::Var("ipsum"))));
            assert_eq!(t.next(), None);
        }

        #[test]
        fn var_lit() {
            let mut t = Tokenizer::new("{{lorem}}ipsum");
            assert_eq!(t.next(), Some(Ok(Token::Var("lorem"))));
            assert_eq!(t.next(), Some(Ok(Token::Lit("ipsum"))));
            assert_eq!(t.next(), None);
        }

        #[test]
        fn edrb_var() {
            let mut t = Tokenizer::new("{{lorem{{ipsum}}");
            assert_eq!(t.next(), Some(Err(Error::ExpectedDoubleRightBraces(7))));
            assert_eq!(t.next(), Some(Ok(Token::Lit("{{lorem"))));
            assert_eq!(t.next(), Some(Ok(Token::Var("ipsum"))));
            assert_eq!(t.next(), None);
        }

        #[test]
        fn edrb_var_edrb_var() {
            let mut t = Tokenizer::new("{{lorem{{ipsum}}{{dolor{{sit}}");
            assert_eq!(t.next(), Some(Err(Error::ExpectedDoubleRightBraces(7))));
            assert_eq!(t.next(), Some(Ok(Token::Lit("{{lorem"))));
            assert_eq!(t.next(), Some(Ok(Token::Var("ipsum"))));
            assert_eq!(t.next(), Some(Err(Error::ExpectedDoubleRightBraces(23))));
            assert_eq!(t.next(), Some(Ok(Token::Lit("{{dolor"))));
            assert_eq!(t.next(), Some(Ok(Token::Var("sit"))));
            assert_eq!(t.next(), None);
        }

        #[test]
        fn edrb_lit() {
            let mut t = Tokenizer::new("{{lorem}ipsum");
            assert_eq!(t.next(), Some(Err(Error::ExpectedDoubleRightBraces(7))));
            assert_eq!(t.next(), Some(Ok(Token::Lit("{{lorem}ipsum"))));
            assert_eq!(t.next(), None);
        }

        #[test]
        fn ueof_lit() {
            let mut t = Tokenizer::new("{{lorem");
            assert_eq!(t.next(), Some(Err(Error::UnexpectedEndOfFile)));
            assert_eq!(t.next(), Some(Ok(Token::Lit("{{lorem"))));
            assert_eq!(t.next(), None);
        }
    }
}

pub use token::Error;
pub use token::Token;

pub fn parse(corpus: &str) -> Result<Vec<Token<'_>>, (Error, Vec<Error>)> {
    let mut result = Ok(vec![]);
    for token in token::Tokenizer::new(corpus) {
        result = match (result, token) {
            (Ok(mut result), Ok(token)) => {
                result.push(token);
                Ok(result)
            }
            (Ok(_), Err(e)) => Err((e, vec![])),
            (Err(result), Ok(_)) => Err(result),
            (Err((first, mut rest)), Err(e)) => {
                rest.push(e);
                Err((first, rest))
            }
        };
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let res = parse("");

        assert_eq!(res, Ok(vec![]));
    }

    #[test]
    fn lit() {
        let res = parse("lorem");

        assert_eq!(res, Ok(vec![Token::Lit("lorem")]));
    }

    #[test]
    fn var() {
        let res = parse("{{lorem}}");

        assert_eq!(res, Ok(vec![Token::Var("lorem")]));
    }

    #[test]
    fn var_var() {
        let res = parse("{{lorem}}{{ipsum}}");

        assert_eq!(res, Ok(vec![Token::Var("lorem"), Token::Var("ipsum")]));
    }

    #[test]
    fn edrb_var() {
        let res = parse("{{lorem{{ipsum}}");

        assert_eq!(res, Err((Error::ExpectedDoubleRightBraces(7), vec![])));
    }

    #[test]
    fn edrb_var_edrb_var() {
        let res = parse("{{lorem{{ipsum}}{{dolor{{sit}}");

        assert_eq!(
            res,
            Err((
                Error::ExpectedDoubleRightBraces(7),
                vec![Error::ExpectedDoubleRightBraces(23)]
            ))
        );
    }
}
