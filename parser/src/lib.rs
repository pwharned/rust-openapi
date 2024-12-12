use std::path::PathBuf;
use std::sync::Arc;
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub dtype: String,
    pub default: Option<String>,
    pub not_null: bool,
    pub options: Vec<String>,
}
#[derive(Debug, Clone)]
pub enum Constraint {
    PrimaryKey(PrimaryKey),
    ForeignKey(ForeignKey),
    Unique(Unique),
}

#[derive(Debug, Clone)]
pub struct Unique {
    pub columns: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct PrimaryKey {
    pub columns: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ForeignKey {
    pub source_columns: Vec<String>,
    pub target_table: String,
    pub target_columns: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub constraints: Vec<String>,
}

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

    pub fn zero_or_more(self) -> Parser<'a, Vec<Output>>
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

    pub fn map<B: 'a, F>(self, f: F) -> Parser<'a, B>
    where
        F: 'a + Send + Sync + Fn(Output) -> B,
    {
        Parser::new(move |input| {
            self.parse(input)
                .map(|(output, remaining_input)| (f(output), remaining_input))
        })
    }

    pub fn and_then<B: 'a, F>(self, f: F) -> Parser<'a, B>
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

    pub fn or(self, other: Parser<'a, Output>) -> Self {
        Parser::new(move |input| self.parse(input).or_else(|| other.parse(input)))
    }
}

pub fn whitespace<'a>() -> Parser<'a, ()> {
    Parser::new(|input: &'a str| {
        let replaced = input.replace("/n", " ");
        let trimmed = replaced.trim_start();

        let len = input.len() - trimmed.len();
        if len > 0 {
            Some(((), &input[len..]))
        } else {
            Some(((), input))
        }
    })
}

pub fn with_whitespace<'a, Output: 'a + Sync + Send>(
    parser: Parser<'a, Output>,
) -> Parser<'a, Output> {
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

pub fn match_char<'a>(expected: char) -> Parser<'a, char> {
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

pub fn match_string<'a>(expected: &'a str) -> Parser<'a, &'a str> {
    Parser::new(move |input: &'a str| {
        if input.to_lowercase().starts_with(&expected.to_lowercase()) {
            return Some((expected, &input[expected.len()..]));
        }
        None
    })
}

pub fn name<'a>() -> Parser<'a, &'a str> {
    Parser::new(|input: &'a str| {
        let chars = input.chars();
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

pub fn primary_key<'a>() -> Parser<'a, Constraint> {
    with_whitespace(match_string("PRIMARY KEY")).and_then(move |_| {
        with_whitespace(match_char('('))
            .and_then(|_| comma_sep(with_whitespace(name())))
            .and_then({
                move |defs| {
                    match_char(')').map(move |_| {
                        Constraint::PrimaryKey(PrimaryKey {
                            columns: defs.to_vec().into_iter().map(|s| s.to_string()).collect(),
                        })
                    })
                }
            })
    })
}
pub fn unique<'a>() -> Parser<'a, Constraint> {
    with_whitespace(match_string("UNIQUE")).and_then(move |_| {
        with_whitespace(match_char('('))
            .and_then(|_| comma_sep(with_whitespace(name())))
            .and_then({
                move |defs| {
                    match_char(')').map(move |_| {
                        Constraint::Unique(Unique {
                            columns: defs.to_vec().into_iter().map(|s| s.to_string()).collect(),
                        })
                    })
                }
            })
    })
}

pub fn constraint_parser<'a>() -> Parser<'a, Constraint> {
    with_whitespace(match_string("CONSTRAINT"))
        .and_then({ move |_| foreign_key().or(primary_key()) })
}
pub fn foreign_key<'a>() -> Parser<'a, Constraint> {
    with_whitespace(match_string("FOREIGN KEY")).and_then(move |_| {
        with_whitespace(match_char('('))
            .and_then(|_| comma_sep(with_whitespace(name())))
            .and_then({
                move |defs| {
                    match_char(')').and_then(move |_| {
                        with_whitespace(match_string("REFERENCES")).and_then({
                            let defs2 = defs.clone();
                            move |_| {
                                with_whitespace(name()).and_then({
                                    let def3 = defs2.clone();
                                    move |tablename| {
                                        with_whitespace(match_char('(')).and_then({
                                            let def4 = def3.clone();
                                            move |_| {
                                                comma_sep(with_whitespace(name())).and_then({
                                                    let def5 = def4.clone();
                                                    move |columnnames| {
                                                        match_char(')').map({
                                                            let def6 = def5.clone();
                                                            move |_| {
                                                                Constraint::ForeignKey(ForeignKey {
                                                                    source_columns: def6
                                                                        .to_vec()
                                                                        .into_iter()
                                                                        .map(|s| s.to_string())
                                                                        .collect::<Vec<String>>(),
                                                                    target_table: tablename
                                                                        .to_string(),
                                                                    target_columns: columnnames
                                                                        .to_vec()
                                                                        .into_iter()
                                                                        .map(|s| s.to_string())
                                                                        .collect::<Vec<String>>(),
                                                                })
                                                            }
                                                        })
                                                    }
                                                })
                                            }
                                        })
                                    }
                                })
                            }
                        })
                    })
                }
            })
    })
}

fn column_def<'a>() -> Parser<'a, Column> {
    with_whitespace(name()).and_then(|colname| {
        with_whitespace(name()).and_then(move |dtype| {
            // Capture default value if present
            let default_parser = with_whitespace(match_string("DEFAULT"))
                .and_then(|_| whitespace().and_then(|_| until().map(|val| Some(val.to_string()))))
                .or(Parser::new(|input| Some((None, input)))); // Changed to return Option<String> wrapped in a tuple

            default_parser.and_then(move |default_value| {
                // Capture NOT NULL constraint if present
                let not_null_parser = with_whitespace(match_string("NOT NULL"))
                    .map(|_| true)
                    .or(Parser::new(|input| Some((false, input)))); // Changed to return (bool, &str)

                not_null_parser.and_then({
                    let value = default_value.clone();
                    move |not_null| {
                        // Capture other constraints
                        let constraint_parser = with_whitespace(match_string("PRIMARY KEY"))
                            .or(with_whitespace(match_string("UNIQUE")))
                            .zero_or_more();

                        constraint_parser.map({
                            let value_final = value.clone();
                            move |constraints| Column {
                                name: colname.to_string(),
                                dtype: dtype.to_string(),
                                default: value_final.clone(), // No need to clone here
                                not_null,
                                options: constraints.into_iter().map(|s| s.to_string()).collect(),
                            }
                        })
                    }
                })
            })
        })
    })
}

pub fn column_list<'a>() -> Parser<'a, Arc<Vec<Column>>> {
    with_whitespace(match_char('('))
        .and_then(|_| comma_sep(column_def()))
        .and_then(move |cols| with_whitespace(match_char(')')).map(move |_| Arc::clone(&cols)))
}
pub fn comma_sep<'a, Output: 'a>(parser: Parser<'a, Output>) -> Parser<'a, Arc<Vec<Output>>> {
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

pub fn create_table_parser<'a>() -> Parser<'a, Table> {
    with_whitespace(match_string("CREATE TABLE"))
        .and_then(move |_| {
            with_whitespace(name())
                .and_then(|_| {
                    with_whitespace(match_char('.')).and_then(|_| with_whitespace(name()))
                })
                .or(with_whitespace(name()))
        })
        .and_then(move |table_name| {
            whitespace().and_then(move |_| {
                column_list().and_then(move |columns| {
                    whitespace().and_then(move |_| {
                        // Capture table constraints
                        //
                        let columns_arc: Arc<Vec<Column>> = Arc::clone(&columns); // Explicit type
                        let constraint_parser = with_whitespace(match_string("CONSTRAINT"))
                            .and_then(|_| with_whitespace(name()))
                            .and_then(move |constraint_name| {
                                until().map(move |definition| {
                                    format!("{} {}", constraint_name, definition)
                                })
                            })
                            .zero_or_more();

                        constraint_parser.map(move |constraints| Table {
                            name: table_name.to_string(),
                            columns: columns_arc.to_vec(),
                            constraints: constraints.into_iter().map(|s| s.to_string()).collect(),
                        })
                    })
                })
            })
        })
}
