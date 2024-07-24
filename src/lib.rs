mod ast;
pub mod document;
pub mod examples;
mod parse;
mod parser;
pub mod str_tagged;
pub mod syntax;
pub use ast::DocumentSyntaxTree;
pub use ast::RawSexp;
pub use ast::Sexp;
pub use ast::SyntaxTree;
pub use parse::ParseError;
pub use parser::Parser;
pub use syntax::Encloser;
pub use syntax::Operator;
pub use syntax::SyntaxContext;
pub use syntax::SyntaxGraph;

#[cfg(test)]
mod core_tests {
  use crate::{
    ast::RawSexp,
    document::Document,
    str_tagged::{
      StringTaggedEncloser, StringTaggedOperator, StringTaggedSyntaxGraph,
    },
    syntax::EncloserOrOperator,
    DocumentSyntaxTree, ParseError, Parser,
  };

  fn leaf(s: String) -> RawSexp {
    RawSexp::leaf(s)
  }

  fn inner(subexpressions: Vec<RawSexp>) -> RawSexp {
    RawSexp::inner(subexpressions)
  }

  fn sexp_graph<'g>() -> StringTaggedSyntaxGraph<'g> {
    StringTaggedSyntaxGraph::contextless_from_descriptions(
      vec![' ', '\n', '\t', '\r'],
      Some('\\'),
      vec![("", "(", ")")],
      vec![],
    )
  }

  fn plus_sexp_graph<'g>() -> StringTaggedSyntaxGraph<'g> {
    StringTaggedSyntaxGraph::contextless_from_descriptions(
      vec![' ', '\n', '\t', '\r'],
      Some('\\'),
      vec![("", "(", ")")],
      vec![("PLUS", "+", 1, 1)],
    )
  }

  fn pipe_sexp_graph<'g>() -> StringTaggedSyntaxGraph<'g> {
    StringTaggedSyntaxGraph::contextless_from_descriptions(
      vec![' ', '\n', '\t', '\r'],
      None,
      vec![("", "(", ")"), ("PIPE", "|", "|")],
      vec![],
    )
  }

  fn quote_sexp_graph<'g>() -> StringTaggedSyntaxGraph<'g> {
    StringTaggedSyntaxGraph::contextless_from_descriptions(
      vec![' ', '\n', '\t', '\r'],
      None,
      vec![("", "(", ")")],
      vec![("QUOTE", "'", 0, 1)],
    )
  }

  fn string_sexp_graph<'g>() -> StringTaggedSyntaxGraph<'g> {
    StringTaggedSyntaxGraph::from_descriptions(
      "root",
      vec![
        (
          "root",
          vec!["", "STRING"],
          None,
          vec![' ', '\n', '\t', '\r'],
        ),
        ("string", vec![], Some('\\'), vec![]),
      ],
      vec![("", "(", ")", "root"), ("STRING", "\"", "\"", "string")],
      vec![],
    )
  }

  fn multi_bracket_graph<'g>() -> StringTaggedSyntaxGraph<'g> {
    StringTaggedSyntaxGraph::contextless_from_descriptions(
      vec![' ', '\n', '\t', '\r'],
      None,
      vec![
        ("", "(", ")"),
        (":SQUARE", "[", "]"),
        (":CURLY", "{", "}"),
        (":HASH_CURLY", "#{", "}"),
      ],
      vec![],
    )
  }

  #[test]
  fn sexp_terminal() {
    assert_eq!(
      Parser::new(sexp_graph(), "hello!").read_next_sexp(),
      Ok(Some(leaf("hello!".to_string())))
    );
  }

  #[test]
  fn sexp_whitespaced_list() {
    assert_eq!(
      Parser::new(sexp_graph(), "( + 1 2 )").read_next_sexp(),
      Ok(Some(inner(vec![
        leaf("+".to_string()),
        leaf("1".to_string()),
        leaf("2".to_string())
      ])))
    );
  }

  #[test]
  fn sexp_list() {
    assert_eq!(
      Parser::new(sexp_graph(), "(1)").read_next_sexp(),
      Ok(Some(inner(vec![leaf("1".to_string())])))
    );
  }

  #[test]
  fn sexp_terminal_non_whitespaced_into_opener() {
    assert_eq!(
      Parser::new(sexp_graph(), "(hello?())").read_next_sexp(),
      Ok(Some(inner(vec![leaf("hello?".to_string()), inner(vec![])])))
    );
  }

  #[test]
  fn sexp_nested_list() {
    assert_eq!(
      Parser::new(sexp_graph(), "(+ 1 (* 2 3))").read_next_sexp(),
      Ok(Some(inner(vec![
        leaf("+".to_string()),
        leaf("1".to_string()),
        inner(vec![
          leaf("*".to_string()),
          leaf("2".to_string()),
          leaf("3".to_string())
        ]),
      ])))
    );
  }

  #[test]
  fn unclosed_list_causes_error() {
    assert_eq!(
      Parser::new(sexp_graph(), "(+ 1 2").read_next_sexp(),
      Err(ParseError::EndOfTextWithOpenEncloser("(".to_string()))
    );
  }

  #[test]
  fn square_bracket() {
    assert_eq!(
      Parser::new(multi_bracket_graph(), "[1 2]").read_next_sexp(),
      Ok(Some(inner(vec![
        leaf(":SQUARE".to_string()),
        leaf("1".to_string()),
        leaf("2".to_string())
      ])))
    );
  }

  #[test]
  fn nested_brackets() {
    assert_eq!(
      Parser::new(multi_bracket_graph(), "([{#{hello!}}])").read_next_sexp(),
      Ok(Some(inner(vec![inner(vec![
        leaf(":SQUARE".to_string()),
        inner(vec![
          leaf(":CURLY".to_string()),
          inner(vec![
            leaf(":HASH_CURLY".to_string()),
            leaf("hello!".to_string())
          ]),
        ]),
      ])])))
    );
  }

  #[test]
  fn nested_brackets_extra_hash() {
    assert_eq!(
      Parser::new(multi_bracket_graph(), "([{####{hello!}}])").read_next_sexp(),
      Ok(Some(inner(vec![inner(vec![
        leaf(":SQUARE".to_string()),
        inner(vec![
          leaf(":CURLY".to_string()),
          leaf("###".to_string()),
          inner(vec![
            leaf(":HASH_CURLY".to_string()),
            leaf("hello!".to_string())
          ]),
        ]),
      ])])))
    );
  }

  #[test]
  fn mismatched_brackets_cause_error() {
    assert_eq!(
      Parser::new(multi_bracket_graph(), "([)]").read_next_sexp(),
      Err(ParseError::UnexpectedCloser(")".to_string()))
    );
  }

  #[test]
  fn prefix_op() {
    assert_eq!(
      Parser::new(quote_sexp_graph(), "'hello!").read_next_sexp(),
      Ok(Some(inner(vec![
        leaf("QUOTE".to_string()),
        leaf("hello!".to_string())
      ])))
    );
  }

  #[test]
  fn prefix_op_in_list() {
    assert_eq!(
      Parser::new(quote_sexp_graph(), "('hello! goodbye!)").read_next_sexp(),
      Ok(Some(inner(vec![
        inner(vec![leaf("QUOTE".to_string()), leaf("hello!".to_string())]),
        leaf("goodbye!".to_string())
      ])))
    );
  }

  #[test]
  fn top_level_infix_op() {
    assert_eq!(
      Parser::new(plus_sexp_graph(), "1+2").read_next_sexp(),
      Ok(Some(inner(vec![
        leaf("PLUS".to_string()),
        leaf("1".to_string()),
        leaf("2".to_string())
      ])))
    );
  }

  #[test]
  fn solo_infix_op_in_list() {
    assert_eq!(
      Parser::new(plus_sexp_graph(), "(1+2)").read_next_sexp(),
      Ok(Some(inner(vec![inner(vec![
        leaf("PLUS".to_string()),
        leaf("1".to_string()),
        leaf("2".to_string())
      ])])))
    );
  }

  #[test]
  fn nested_infix_op_in_list() {
    assert_eq!(
      Parser::new(plus_sexp_graph(), "(1+2+3)").read_next_sexp(),
      Ok(Some(inner(vec![inner(vec![
        leaf("PLUS".to_string()),
        inner(vec![
          leaf("PLUS".to_string()),
          leaf("1".to_string()),
          leaf("2".to_string())
        ]),
        leaf("3".to_string())
      ])])))
    );
  }

  #[test]
  fn terminals_after_infix_op_in_list() {
    assert_eq!(
      Parser::new(plus_sexp_graph(), "(1+2 3)").read_next_sexp(),
      Ok(Some(inner(vec![
        inner(vec![
          leaf("PLUS".to_string()),
          leaf("1".to_string()),
          leaf("2".to_string())
        ]),
        leaf("3".to_string())
      ])))
    );
  }

  #[test]
  fn op_missing_left_arg_causes_error() {
    assert_eq!(
      Parser::new(plus_sexp_graph(), "(+2)").read_next_sexp(),
      Err(ParseError::OperatorMissingLeftArgument("+".to_string()))
    );
  }

  #[test]
  fn unfinished_infix_op_causes_error() {
    assert_eq!(
      Parser::new(plus_sexp_graph(), "(1+)").read_next_sexp(),
      Err(ParseError::OperatorMissingRightArgument("+".to_string()))
    );
  }

  #[test]
  fn unfinished_top_level_infix_op_causes_error() {
    assert_eq!(
      Parser::new(plus_sexp_graph(), "1+").read_next_sexp(),
      Err(ParseError::OperatorMissingRightArgument("+".to_string()))
    );
  }

  #[test]
  fn contextful_brackets() {
    assert_eq!(
      Parser::new(
        StringTaggedSyntaxGraph::from_descriptions(
          "root",
          vec![
            (
              "root",
              vec!["", "SQUARE"],
              None,
              vec![' ', '\n', '\t', '\r'],
            ),
            (
              "include_angle",
              vec!["", "SQUARE", "ANGLE"],
              None,
              vec![' ', '\n', '\t', '\r'],
            )
          ],
          vec![
            ("", "(", ")", "root"),
            ("SQUARE", "[", "]", "include_angle"),
            ("ANGLE", "<", ">", "include_angle")
          ],
          vec![]
        ),
        "(> < [<>])"
      )
      .read_next_sexp(),
      Ok(Some(inner(vec![
        leaf(">".to_string()),
        leaf("<".to_string()),
        inner(vec![
          leaf("SQUARE".to_string()),
          inner(vec![leaf("ANGLE".to_string())])
        ]),
      ])))
    );
  }

  #[test]
  fn contextful_operator() {
    assert_eq!(
      Parser::new(
        StringTaggedSyntaxGraph::from_descriptions(
          "root",
          vec![
            ("root", vec!["", "COLON"], None, vec![' ', '\n', '\t', '\r'],),
            (
              "include_angle",
              vec!["", "ANGLE", "COLON"],
              None,
              vec![' ', '\n', '\t', '\r'],
            )
          ],
          vec![("", "(", ")", "root"), ("ANGLE", "<", ">", "include_angle")],
          vec![("COLON", ":", 1, 1, "include_angle")],
        ),
        "((> 1 0) : <Bool>)"
      )
      .read_next_sexp(),
      Ok(Some(inner(vec![inner(vec![
        leaf("COLON".to_string()),
        inner(vec![
          leaf(">".to_string()),
          leaf("1".to_string()),
          leaf("0".to_string())
        ]),
        inner(vec![leaf("ANGLE".to_string()), leaf("Bool".to_string())])
      ])])))
    );
  }

  #[test]
  fn symmetric_encloser() {
    assert_eq!(
      Parser::new(pipe_sexp_graph(), "|+ 1 2|").read_next_sexp(),
      Ok(Some(inner(vec![
        leaf("PIPE".to_string()),
        leaf("+".to_string()),
        leaf("1".to_string()),
        leaf("2".to_string())
      ])))
    );
  }

  #[test]
  fn escaped_closer() {
    assert_eq!(
      Parser::new(sexp_graph(), "(\\))").read_next_sexp(),
      Ok(Some(inner(vec![leaf("\\)".to_string())])))
    );
  }

  #[test]
  fn escaped_opener() {
    assert_eq!(
      Parser::new(sexp_graph(), "(\\()").read_next_sexp(),
      Ok(Some(inner(vec![leaf("\\(".to_string())])))
    );
  }

  #[test]
  fn escaped_operator() {
    assert_eq!(
      Parser::new(plus_sexp_graph(), "(\\+)").read_next_sexp(),
      Ok(Some(inner(vec![leaf("\\+".to_string())])))
    );
  }

  #[test]
  fn symmetric_enclosers_in_list() {
    assert_eq!(
      Parser::new(pipe_sexp_graph(), "(|+ 1 2| |a|)").read_next_sexp(),
      Ok(Some(inner(vec![
        inner(vec![
          leaf("PIPE".to_string()),
          leaf("+".to_string()),
          leaf("1".to_string()),
          leaf("2".to_string())
        ]),
        inner(vec![leaf("PIPE".to_string()), leaf("a".to_string())])
      ])))
    );
  }

  #[test]
  fn nested_symmetric_enclosers() {
    assert_eq!(
      Parser::new(pipe_sexp_graph(), "|(|a|)|").read_next_sexp(),
      Ok(Some(inner(vec![
        leaf("PIPE".to_string()),
        inner(vec![inner(vec![
          leaf("PIPE".to_string()),
          leaf("a".to_string())
        ])])
      ])))
    );
  }

  #[test]
  fn read_two_sexps() {
    let mut parser = Parser::new(sexp_graph(), "(+ 1 2) (* 3 4)");
    assert_eq!(
      parser.read_next_sexp(),
      Ok(Some(inner(vec![
        leaf("+".to_string()),
        leaf("1".to_string()),
        leaf("2".to_string())
      ])))
    );
    assert_eq!(
      parser.read_next_sexp(),
      Ok(Some(inner(vec![
        leaf("*".to_string()),
        leaf("3".to_string()),
        leaf("4".to_string())
      ])))
    );
  }

  #[test]
  fn read_all_single_sexp() {
    assert_eq!(
      Parser::new(sexp_graph(), "(+ 1 2)").read_all_sexps(),
      vec![Ok(inner(vec![
        leaf("+".to_string()),
        leaf("1".to_string()),
        leaf("2".to_string())
      ]))]
    );
  }

  #[test]
  fn read_all_double_sexp() {
    assert_eq!(
      Parser::new(sexp_graph(), "(+ 1 2) (* 3 4)").read_all_sexps(),
      vec![
        Ok(inner(vec![
          leaf("+".to_string()),
          leaf("1".to_string()),
          leaf("2".to_string())
        ])),
        Ok(inner(vec![
          leaf("*".to_string()),
          leaf("3".to_string()),
          leaf("4".to_string())
        ]))
      ]
    );
  }

  #[test]
  fn read_all_double_sexp_err() {
    assert_eq!(
      Parser::new(sexp_graph(), "(+ 1 2) (* 3 4").read_all_sexps(),
      vec![
        Ok(inner(vec![
          leaf("+".to_string()),
          leaf("1".to_string()),
          leaf("2".to_string())
        ])),
        Err(ParseError::EndOfTextWithOpenEncloser("(".to_string()))
      ]
    );
  }

  #[test]
  fn contextful_whitespace() {
    assert_eq!(
      Parser::new(
        string_sexp_graph(),
        "(before string \" inside string!!! \" after string)"
      )
      .read_next_sexp(),
      Ok(Some(inner(vec![
        leaf("before".to_string()),
        leaf("string".to_string()),
        inner(vec![
          leaf("STRING".to_string()),
          leaf(" inside string!!! ".to_string()),
        ]),
        leaf("after".to_string()),
        leaf("string".to_string()),
      ])))
    );
  }

  #[test]
  fn contextful_escape() {
    assert_eq!(
      Parser::new(string_sexp_graph(), "\"\\\"\"").read_next_sexp(),
      Ok(Some(inner(vec![
        leaf("STRING".to_string()),
        leaf("\\\"".to_string()),
      ])))
    );
  }

  #[test]
  fn solo_sexp_char_indeces() {
    assert_eq!(
      Parser::new(sexp_graph(), "(+ 1 2)").read_next(),
      Ok(Some(DocumentSyntaxTree::Inner(
        (
          0..7,
          EncloserOrOperator::Encloser(StringTaggedEncloser::new("", "(", ")"))
        ),
        vec![
          DocumentSyntaxTree::Leaf(1..2, "+".to_string()),
          DocumentSyntaxTree::Leaf(3..4, "1".to_string()),
          DocumentSyntaxTree::Leaf(5..6, "2".to_string())
        ]
      )))
    )
  }

  #[test]
  fn nested_sexp_char_indeces() {
    assert_eq!(
      Parser::new(sexp_graph(), "(* (+ 1 2) 3)").read_next(),
      Ok(Some(DocumentSyntaxTree::Inner(
        (
          0..13,
          EncloserOrOperator::Encloser(StringTaggedEncloser::new("", "(", ")"))
        ),
        vec![
          DocumentSyntaxTree::Leaf(1..2, "*".to_string()),
          DocumentSyntaxTree::Inner(
            (
              3..10,
              EncloserOrOperator::Encloser(StringTaggedEncloser::new(
                "", "(", ")"
              ))
            ),
            vec![
              DocumentSyntaxTree::Leaf(4..5, "+".to_string()),
              DocumentSyntaxTree::Leaf(6..7, "1".to_string()),
              DocumentSyntaxTree::Leaf(8..9, "2".to_string())
            ]
          ),
          DocumentSyntaxTree::Leaf(11..12, "3".to_string()),
        ]
      )))
    )
  }

  #[test]
  fn multi_bracket_sexp_char_indeces() {
    assert_eq!(
      Parser::new(multi_bracket_graph(), "(union #{1 20} #{})").read_next(),
      Ok(Some(DocumentSyntaxTree::Inner(
        (
          0..19,
          EncloserOrOperator::Encloser(StringTaggedEncloser::new("", "(", ")"))
        ),
        vec![
          DocumentSyntaxTree::Leaf(1..6, "union".to_string()),
          DocumentSyntaxTree::Inner(
            (
              7..14,
              EncloserOrOperator::Encloser(StringTaggedEncloser::new(
                ":HASH_CURLY",
                "#{",
                "}"
              ))
            ),
            vec![
              DocumentSyntaxTree::Leaf(9..10, "1".to_string()),
              DocumentSyntaxTree::Leaf(11..13, "20".to_string())
            ]
          ),
          DocumentSyntaxTree::Inner(
            (
              15..18,
              EncloserOrOperator::Encloser(StringTaggedEncloser::new(
                ":HASH_CURLY",
                "#{",
                "}"
              ))
            ),
            vec![]
          ),
        ]
      )))
    )
  }

  #[test]
  fn infix_char_indeces() {
    assert_eq!(
      Parser::new(plus_sexp_graph(), "(1+2)").read_next(),
      Ok(Some(DocumentSyntaxTree::Inner(
        (
          0..5,
          EncloserOrOperator::Encloser(StringTaggedEncloser::new("", "(", ")"))
        ),
        vec![DocumentSyntaxTree::Inner(
          (
            1..4,
            EncloserOrOperator::Operator(StringTaggedOperator::new(
              "PLUS", "+", 1, 1
            ))
          ),
          vec![
            DocumentSyntaxTree::Leaf(1..2, "1".to_string()),
            DocumentSyntaxTree::Leaf(3..4, "2".to_string())
          ]
        ),]
      )))
    );
  }

  #[test]
  fn sexp_document_subtree() {
    let doc: Document<_, _> = Parser::new(sexp_graph(), "(* (+ 1 2) 3)")
      .try_into()
      .unwrap();
    assert_eq!(
      doc.get_subtree(&[0]).unwrap().clone(),
      Parser::new(sexp_graph(), "(* (+ 1 2) 3)")
        .read_next()
        .unwrap()
        .unwrap()
    );
    assert_eq!(
      RawSexp::from(doc.get_subtree(&[0, 0]).unwrap().clone()),
      Parser::new(sexp_graph(), "*")
        .read_next_sexp()
        .unwrap()
        .unwrap()
    );
    assert_eq!(
      RawSexp::from(doc.get_subtree(&[0, 1]).unwrap().clone()),
      Parser::new(sexp_graph(), "(+ 1 2)")
        .read_next_sexp()
        .unwrap()
        .unwrap()
    );
  }

  #[test]
  fn infix_sexp_document_subtree() {
    let doc: Document<_, _> = Parser::new(plus_sexp_graph(), "(inc 1 + 2)")
      .try_into()
      .unwrap();
    assert_eq!(
      RawSexp::from(doc.get_subtree(&[0, 0]).unwrap().clone()),
      Parser::new(plus_sexp_graph(), "inc")
        .read_next_sexp()
        .unwrap()
        .unwrap()
    );
    assert_eq!(
      RawSexp::from(doc.get_subtree(&[0, 1]).unwrap().clone()),
      Parser::new(plus_sexp_graph(), "1 + 2")
        .read_next_sexp()
        .unwrap()
        .unwrap()
    );
    assert_eq!(
      RawSexp::from(doc.get_subtree(&[0, 1, 0]).unwrap().clone()),
      Parser::new(plus_sexp_graph(), "1")
        .read_next_sexp()
        .unwrap()
        .unwrap()
    );
    assert_eq!(
      RawSexp::from(doc.get_subtree(&[0, 1, 1]).unwrap().clone()),
      Parser::new(plus_sexp_graph(), "2")
        .read_next_sexp()
        .unwrap()
        .unwrap()
    );
  }

  #[test]
  fn sexp_document_enclosing_paths() {
    let doc: Document<_, _> = Parser::new(sexp_graph(), "(* (+ 1 2) 3)")
      .try_into()
      .unwrap();
    assert_eq!(doc.innermost_enclosing_path(&(0..0)), vec![0]);
    assert_eq!(doc.innermost_enclosing_path(&(1..1)), vec![0, 0]);
    assert_eq!(doc.innermost_enclosing_path(&(2..2)), vec![0, 0]);
    assert_eq!(doc.innermost_enclosing_path(&(3..3)), vec![0, 1]);
    assert_eq!(doc.innermost_enclosing_path(&(4..4)), vec![0, 1, 0]);
    assert_eq!(doc.innermost_enclosing_path(&(5..5)), vec![0, 1, 0]);
    assert_eq!(doc.innermost_enclosing_path(&(6..6)), vec![0, 1, 1]);
    assert_eq!(doc.innermost_enclosing_path(&(4..6)), vec![0, 1]);
    assert_eq!(doc.innermost_enclosing_path(&(5..6)), vec![0, 1]);
    assert_eq!(doc.innermost_enclosing_path(&(0..2)), vec![0]);
    assert_eq!(doc.innermost_enclosing_path(&(100..200)), vec![]);
  }
}
