use parse::{
    cascade, column, comma_sep, constraint, create_table_parser, foreign_key, function, match_char,
    match_string, name, with_whitespace, ForeignKey, ParseError, Parser,
};
use std::sync::Arc;
mod tests {
    use super::*;

    use std::fs;

    fn process_reult<'a, A>(res: Result<A, ParseError>)
    where
        A: 'a + std::fmt::Debug,
    {
        match res {
            Ok(a) => println!("{:?}", a),
            Err(e) => panic!("{:?}", e),
        }
    }

    use std::path::PathBuf;
    #[test]
    fn testSelectParser() {
        let select_parser = with_whitespace(match_string("SELECT"))
            .and_then(|_| with_whitespace(match_string("*")));
        let result = select_parser.parse(" SELECT   * ");

        process_reult(result);
    }

    #[test]
    fn test_constraint_parser() {
        let constraints = "CONSTRAINT asset_types_pkey PRIMARY KEY (type_id) ";
        process_reult(constraint().or(column()).parse(constraints))
    }
    #[test]
    fn testOrParser() {
        let or = match_string("HELLO")
            .or(match_string("GOODBYE"))
            .or(match_string("FOO"));
    }

    #[test]
    fn test_comma() {
        let commavals = "FOREIGN KEY (HELLO,GOODBYE ) REFERENCES TABLE(TEST, TEST)";
        let column_parser = with_whitespace(match_char('('))
            .and_then(|_| comma_sep(with_whitespace(parse::name())))
            .and_then({ move |defs| match_char(')').map({ move |_| defs.clone() }) });

        let references_parser = with_whitespace(match_string("REFERENCES"));

        let result = foreign_key().parse(commavals);
        match result {
            Ok(a) => println!("{:?}", a),
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn default_test() {
        let default_parser = with_whitespace(match_string("DEFAULT"))
            .and_then(|_| {
                with_whitespace(function())
                    .or(with_whitespace(name()))
                    .map(|val| Ok(val.to_string()))
            })
            .or(Parser::new(|input| Ok((Err("No default value"), input))));

        process_reult(default_parser.parse("DEFAULT gen_random_uuid() NOT NULL"));
    }

    #[test]
    fn function_parser() {
        let input = "get_random_uuid()";

        let result = function().parse(input);
        process_reult(result);
    }

    #[test]
    fn name_parser() {
        let input = "\\\"hello\"";
        name().parse(input);
    }

    #[test]
    fn test_cascade() {
        process_reult(cascade().parse("ON DELETE CASCADE"));
        process_reult(cascade().parse("ON DELETE CASCADE ON UPDATE CASCADE"));
        process_reult(cascade().parse("ON UPDATE CASCADE"));
        process_reult(cascade().parse(""));
    }
    #[test]
    fn test_create_table_parser() {
        let result = parse::create_table_parser()
            .parse(" CREATE TABLE TEST ( id int PRIMARY KEY , id2 int NOT NULL ) ");

        process_reult(result);
        let psql = "CREATE TABLE public.collections ( collection_id uuid DEFAULT gen_random_uuid() NOT NULL, collection_name text NOT NULL, created_at timestamptz DEFAULT now() NOT NULL, updated_at timestamptz DEFAULT now() NOT NULL, collection_description text NOT NULL, collection_owner text NOT NULL, collection_collaborators text NULL )";

        println!("{:?}", &psql);
        let pres = create_table_parser().parse(&psql);

        process_reult(pres);
        let relative_path = "../openapi/ddl.sql";

        // Construct the absolute path
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let file_path = PathBuf::from(manifest_dir).join(relative_path);
        // Read the JSON file
        let file_content = fs::read_to_string(file_path).expect("Unable to read file");
        let sql = file_content.split(";");

        for stmt in sql {
            let clean = stmt
                .replace("\n", " ")
                .replace("\t", " ")
                .replace("  ", " ")
                .to_string();
            let parsed = create_table_parser().parse(&clean);
            process_reult(parsed);
        }
        // Generate structs based on the JSON data
    }
}
