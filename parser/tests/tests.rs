use parse::{
    comma_sep, create_table_parser, foreign_key, match_char, match_string, name, with_whitespace,
    ForeignKey,
};
use std::sync::Arc;
mod tests {
    use super::*;
    use std::fs;

    use std::path::PathBuf;
    #[test]
    fn testSelectParser() {
        let select_parser = with_whitespace(match_string("SELECT"))
            .and_then(|_| with_whitespace(match_string("*")));
        let result = select_parser.parse(" SELECT   * ");
        assert_eq!(result, Some(("*", "")));
    }

    #[test]
    fn testOrParser() {
        let or = match_string("HELLO")
            .or(match_string("GOODBYE"))
            .or(match_string("FOO"));
        assert!(or.parse("HELLO") == Some(("HELLO", "")));
        assert!(or.parse("GOODBYE") == Some(("GOODBYE", "")));
        assert!(or.parse("FOO") == Some(("FOO", "")));
    }

    #[test]
    fn test_comma() {
        let commavals = "FOREIGN KEY (HELLO,GOODBYE ) REFERENCES TABLE(TEST, TEST)";
        let column_parser = with_whitespace(match_char('('))
            .and_then(|_| comma_sep(with_whitespace(parse::name())))
            .and_then({ move |defs| match_char(')').map({ move |_| defs.clone() }) });

        let references_parser = with_whitespace(match_string("REFERENCES"));

        let result = foreign_key().parse(commavals);
    }

    #[test]
    fn test_create_table_parser() {
        let create_table_result = parse::create_table_parser()
            .parse(" CREATE TABLE TEST ( id int PRIMARY KEY , id2 int NOT NULL ) ");

        println!("{:?}", create_table_result);

        let psql = "  CREATE TABLE public.asset_types ( type_id text NOT NULL, type_name text NOT NULL, CONSTRAINT asset_types_pkey PRIMARY KEY (type_id), CONSTRAINT asset_types_type_name_key UNIQUE (type_name) )";
        println!("{:?}", &psql);
        let pres = create_table_parser().parse(&psql);
        println!("{:?}", pres);

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
            println!("{:?}", clean);
            println!("{:?}", parsed);
        }
        // Generate structs based on the JSON data
    }
}
