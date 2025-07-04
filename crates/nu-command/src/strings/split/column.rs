use fancy_regex::{escape, Regex};
use nu_engine::command_prelude::*;

#[derive(Clone)]
pub struct SplitColumn;

impl Command for SplitColumn {
    fn name(&self) -> &str {
        "split column"
    }

    fn signature(&self) -> Signature {
        Signature::build("split column")
            .input_output_types(vec![
                (Type::String, Type::table()),
                (
                    // TODO: no test coverage (is this behavior a bug or a feature?)
                    Type::List(Box::new(Type::String)),
                    Type::table(),
                ),
            ])
            .required(
                "separator",
                SyntaxShape::String,
                "The character or string that denotes what separates columns.",
            )
            .switch("collapse-empty", "remove empty columns", Some('c'))
            .named(
                "number",
                SyntaxShape::Int,
                "Split into maximum number of items",
                Some('n'),
            )
            .switch("regex", "separator is a regular expression", Some('r'))
            .rest(
                "rest",
                SyntaxShape::String,
                "Column names to give the new columns.",
            )
            .category(Category::Strings)
    }

    fn description(&self) -> &str {
        "Split a string into multiple columns using a separator."
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["separate", "divide", "regex"]
    }

    fn examples(&'_ self) -> Vec<Example<'_>> {
        vec![
            Example {
                description: "Split a string into columns by the specified separator",
                example: "'a--b--c' | split column '--'",
                result: Some(Value::test_list(vec![Value::test_record(record! {
                        "column1" => Value::test_string("a"),
                        "column2" => Value::test_string("b"),
                        "column3" => Value::test_string("c"),
                })])),
            },
            Example {
                description: "Split a string into columns of char and remove the empty columns",
                example: "'abc' | split column --collapse-empty ''",
                result: Some(Value::test_list(vec![Value::test_record(record! {
                        "column1" => Value::test_string("a"),
                        "column2" => Value::test_string("b"),
                        "column3" => Value::test_string("c"),
                })])),
            },
            Example {
                description: "Split a list of strings into a table",
                example: "['a-b' 'c-d'] | split column -",
                result: Some(Value::test_list(vec![
                    Value::test_record(record! {
                        "column1" => Value::test_string("a"),
                        "column2" => Value::test_string("b"),
                    }),
                    Value::test_record(record! {
                        "column1" => Value::test_string("c"),
                        "column2" => Value::test_string("d"),
                    }),
                ])),
            },
            Example {
                description: "Split a list of strings into a table, ignoring padding",
                example: r"['a -  b' 'c  -    d'] | split column --regex '\s*-\s*'",
                result: Some(Value::test_list(vec![
                    Value::test_record(record! {
                        "column1" => Value::test_string("a"),
                        "column2" => Value::test_string("b"),
                    }),
                    Value::test_record(record! {
                        "column1" => Value::test_string("c"),
                        "column2" => Value::test_string("d"),
                    }),
                ])),
            },
            Example {
                description: "Split into columns, last column may contain the delimiter",
                example: r"['author: Salina Yoon' r#'title: Where's Ellie?: A Hide-and-Seek Book'#] | split column --number 2 ': ' key value",
                result: Some(Value::test_list(vec![
                    Value::test_record(record! {
                        "key" => Value::test_string("author"),
                        "value" => Value::test_string("Salina Yoon"),
                    }),
                    Value::test_record(record! {
                        "key" => Value::test_string("title"),
                        "value" => Value::test_string("Where's Ellie?: A Hide-and-Seek Book"),
                    }),
                ])),
            },
        ]
    }

    fn is_const(&self) -> bool {
        true
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let separator: Spanned<String> = call.req(engine_state, stack, 0)?;
        let rest: Vec<Spanned<String>> = call.rest(engine_state, stack, 1)?;
        let collapse_empty = call.has_flag(engine_state, stack, "collapse-empty")?;
        let max_split: Option<usize> = call.get_flag(engine_state, stack, "number")?;
        let has_regex = call.has_flag(engine_state, stack, "regex")?;

        let args = Arguments {
            separator,
            rest,
            collapse_empty,
            max_split,
            has_regex,
        };
        split_column(engine_state, call, input, args)
    }

    fn run_const(
        &self,
        working_set: &StateWorkingSet,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let separator: Spanned<String> = call.req_const(working_set, 0)?;
        let rest: Vec<Spanned<String>> = call.rest_const(working_set, 1)?;
        let collapse_empty = call.has_flag_const(working_set, "collapse-empty")?;
        let max_split: Option<usize> = call.get_flag_const(working_set, "number")?;
        let has_regex = call.has_flag_const(working_set, "regex")?;

        let args = Arguments {
            separator,
            rest,
            collapse_empty,
            max_split,
            has_regex,
        };
        split_column(working_set.permanent(), call, input, args)
    }
}

struct Arguments {
    separator: Spanned<String>,
    rest: Vec<Spanned<String>>,
    collapse_empty: bool,
    max_split: Option<usize>,
    has_regex: bool,
}

fn split_column(
    engine_state: &EngineState,
    call: &Call,
    input: PipelineData,
    args: Arguments,
) -> Result<PipelineData, ShellError> {
    let name_span = call.head;
    let regex = if args.has_regex {
        Regex::new(&args.separator.item)
    } else {
        let escaped = escape(&args.separator.item);
        Regex::new(&escaped)
    }
    .map_err(|e| ShellError::GenericError {
        error: "Error with regular expression".into(),
        msg: e.to_string(),
        span: Some(args.separator.span),
        help: None,
        inner: vec![],
    })?;

    input.flat_map(
        move |x| {
            split_column_helper(
                &x,
                &regex,
                &args.rest,
                args.collapse_empty,
                args.max_split,
                name_span,
            )
        },
        engine_state.signals(),
    )
}

fn split_column_helper(
    v: &Value,
    separator: &Regex,
    rest: &[Spanned<String>],
    collapse_empty: bool,
    max_split: Option<usize>,
    head: Span,
) -> Vec<Value> {
    if let Ok(s) = v.coerce_str() {
        let split_result: Vec<_> = match max_split {
            Some(max_split) => separator
                .splitn(&s, max_split)
                .filter_map(|x| x.ok())
                .filter(|x| !(collapse_empty && x.is_empty()))
                .collect(),
            None => separator
                .split(&s)
                .filter_map(|x| x.ok())
                .filter(|x| !(collapse_empty && x.is_empty()))
                .collect(),
        };
        let positional: Vec<_> = rest.iter().map(|f| f.item.clone()).collect();

        // If they didn't provide column names, make up our own
        let mut record = Record::new();
        if positional.is_empty() {
            let mut gen_columns = vec![];
            for i in 0..split_result.len() {
                gen_columns.push(format!("column{}", i + 1));
            }

            for (&k, v) in split_result.iter().zip(&gen_columns) {
                record.push(v, Value::string(k, head));
            }
        } else {
            for (&k, v) in split_result.iter().zip(&positional) {
                record.push(v, Value::string(k, head));
            }
        }
        vec![Value::record(record, head)]
    } else {
        match v {
            Value::Error { error, .. } => {
                vec![Value::error(*error.clone(), head)]
            }
            v => {
                let span = v.span();
                vec![Value::error(
                    ShellError::OnlySupportsThisInputType {
                        exp_input_type: "string".into(),
                        wrong_type: v.get_type().to_string(),
                        dst_span: head,
                        src_span: span,
                    },
                    span,
                )]
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(SplitColumn {})
    }
}
