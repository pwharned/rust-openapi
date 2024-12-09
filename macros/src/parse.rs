use std::{process::Output, sync::Arc};

pub type Parse<'a, Output> = Arc<dyn Fn(&'a str) -> Option<(Output, &'a str)> + 'a + Send + Sync>;
pub struct Parser<'a, Output> {
    parser: Arc<dyn Fn(&'a str) -> Option<(Output, &'a str)> + 'a + Send + Sync>,
}

impl<'a, Output: 'a> Parser<'a, Output> {
    // ... existing methods

    fn one_or_more(self) -> Parser<'a, Vec<Output>>
    where
        Output: Clone + 'a,
    {
        Parser::new(move |mut input: &'a str| {
            let mut results = Vec::new();

            // Parse the first occurrence to ensure at least one match
            if let Some((first_result, remaining_input)) = self.parse(input) {
                results.push(first_result);
                input = remaining_input;
            } else {
                return None;
            }

            // Continue parsing while there are more matches
            while let Some((result, remaining_input)) = self.parse(input) {
                results.push(result);
                input = remaining_input;
            }

            Some((results, input))
        })
    }

    fn zero_or_more(self) -> Parser<'a, Vec<Output>>
    where
        Output: Clone + 'a,
    {
        Parser::new(move |mut input: &'a str| {
            let mut results = Vec::new();

            // Parse the first occurrence to ensure at least one match

            // Continue parsing while there are more matches
            while let Some((result, remaining_input)) = self.parse(input) {
                results.push(result);
                input = remaining_input;
            }

            Some((results, input))
        })
    }
}

impl<'a, Output: 'a> Parser<'a, Output> {
    pub fn new<F>(parser: F) -> Self
    where
        F: 'a + Fn(&'a str) -> Option<(Output, &'a str)> + 'a + Send + Sync,
    {
        Self {
            parser: Arc::new(parser),
        }
    }

    pub fn parse(&self, input: &'a str) -> Option<(Output, &'a str)> {
        (self.parser)(input)
    }

    fn map<B: 'a, F>(self, f: F) -> Parser<'a, B>
    where
        F: 'a + Send + Sync + Fn(Output) -> B,
    {
        Parser::new(move |input| {
            self.parse(input)
                .map(|(output, remaining_input)| (f(output), remaining_input))
        })
    }

    fn and_then<B: 'a, F>(self, f: F) -> Parser<'a, B>
    where
        F: 'a + Send + Sync + Fn(Output) -> Parser<'a, B>,
    {
        Parser::new(move |input| {
            if let Some((output1, remaining_input)) = self.parse(input) {
                return f(output1).parse(remaining_input);
            }
            None
        })
    }

    fn or(self, other: Parser<'a, Output>) -> Self {
        Parser::new(move |input| self.parse(input).or_else(|| other.parse(input)))
    }
}

fn whitespace<'a>() -> Parser<'a, ()> {
    Parser::new(|input: &'a str| {
        let trimmed = input.trim_start();
        let len = input.len() - trimmed.len();
        if len > 0 {
            Some(((), &input[len..]))
        } else {
            Some(((), input))
        }
    })
}

fn with_whitespace<'a, Output: 'a + Sync + Send>(parser: Parser<'a, Output>) -> Parser<'a, Output> {
    let parser: Parse<Output> = Arc::clone(&parser.parser);
    whitespace().and_then(move |_| {
        let parser = Arc::clone(&parser);
        Parser::new(move |input| {
            parser(input).and_then(move |(result, remaining_input)| {
                whitespace()
                    .parse(remaining_input)
                    .map(|(_, remaining_input)| (result, remaining_input))
            })
        })
    })
}

fn match_char<'a>(expected: char) -> Parser<'a, char> {
    Parser::new(move |input: &'a str| {
        let mut chars = input.chars();
        if let Some(first_char) = chars.next() {
            if first_char == expected {
                return Some((first_char, chars.as_str()));
            }
        }
        None
    })
}

fn match_string<'a>(expected: &'a str) -> Parser<'a, &'a str> {
    Parser::new(move |input: &'a str| {
        if input.starts_with(expected) {
            return Some((expected, &input[expected.len()..]));
        }
        None
    })
}

fn name<'a>() -> Parser<'a, &'a str> {
    Parser::new(|input: &'a str| {
        let mut chars = input.chars();
        let mut end = 0;
        for c in chars {
            if c.is_alphanumeric() || c == '_' {
                end += c.len_utf8();
            } else {
                break;
            }
        }
        if end > 0 {
            Some((&input[..end], &input[end..]))
        } else {
            None
        }
    })
}

fn until<'a>() -> Parser<'a, &'a str> {
    Parser::new(|input: &'a str| {
        let chars = input.chars();
        let mut end = 0;
        for c in chars {
            if c != ',' {
                end += c.len_utf8();
            } else {
                break;
            }
        }
        if end > 0 {
            Some((&input[..end], &input[end..]))
        } else {
            None
        }
    })
}

fn column_def<'a>() -> Parser<'a, (&'a str, &'a str, Vec<&'a str>)> {
    name().and_then(|colname| {
        whitespace().and_then(move |_| {
            name().and_then(move |dtype| {
                whitespace().and_then(move |_| {
                    with_whitespace(match_string("PRIMARY KEY"))
                        .zero_or_more()
                        .map(move |options| (colname, dtype, options))
                })
            })
        })
    })
}

fn comma_sep<'a, Output: 'a>(parser: Parser<'a, Output>) -> Parser<'a, Arc<Vec<Output>>> {
    Parser::new(move |input: &'a str| {
        let mut result = Vec::new();
        let mut remaining_input = input;
        while let Some((item, rest)) = parser.parse(remaining_input) {
            result.push(item);
            remaining_input = rest;
            if let Some((_, rest)) = with_whitespace(match_char(',')).parse(remaining_input) {
                remaining_input = rest;
            } else {
                break;
            }
        }
        Some((Arc::new(result), remaining_input))
    })
}

fn column_list<'a>() -> Parser<'a, Arc<Arc<Vec<(&'a str, &'a str, Vec<&'a str>)>>>> {
    match_char('(')
        .and_then(|_| comma_sep(column_def()))
        .and_then(|cols| {
            // Clone the Arc before entering the inner closure
            let cols_arc = Arc::new(cols);
            match_char(')').map(move |_| Arc::clone(&cols_arc))
        })
}

pub fn create_table_parser<'a>(
) -> Parser<'a, (String, Arc<Arc<Vec<(&'a str, &'a str, Vec<&'a str>)>>>)> {
    with_whitespace(match_string("CREATE"))
        .and_then(|_| with_whitespace(match_string("TABLE")))
        .and_then(|_| name())
        .and_then(|table_name| column_list().map(move |cols| (table_name.to_string(), cols)))
}

mod tests {
    use super::*;
    #[test]
    fn test() {
        let select_parser = with_whitespace(match_string("SELECT"))
            .and_then(|_| with_whitespace(match_string("*")));
        let result = select_parser.parse(" SELECT   * ");
        println!("{:?}", result);
        assert_eq!(result, Some(("*", "")));

        let create_table_result =
            create_table_parser().parse("CREATE TABLE TEST(id int PRIMARY KEY, id2 int)");

        println!("{:?}", create_table_result);
        let hello_parser = with_whitespace(match_string("HELLO"));
        let name_parser = hello_parser
            .zero_or_more()
            .and_then(move |_| with_whitespace(name()));
        println!("{:?}", name_parser.parse("HELLO GOODBYE"));

        let primary_key_parser =
            with_whitespace(match_string("PRIMARY KEY")).and_then(move |_| with_whitespace(name()));
        let result = primary_key_parser.parse("PRIMARY KEY ID");
        println!("{:?}", result);
    }
}
