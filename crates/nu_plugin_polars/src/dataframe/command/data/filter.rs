use crate::{
    dataframe::values::{Column, NuDataFrame, NuExpression, NuLazyFrame},
    values::CustomValueSupport,
    PolarsPlugin,
};

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, ShellError, Signature, Span, SyntaxShape, Type,
    Value,
};

#[derive(Clone)]
pub struct LazyFilter;

impl PluginCommand for LazyFilter {
    type Plugin = PolarsPlugin;

    fn name(&self) -> &str {
        "polars filter"
    }

    fn description(&self) -> &str {
        "Filter dataframe based in expression."
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "filter expression",
                SyntaxShape::Any,
                "Expression that define the column selection",
            )
            .input_output_type(
                Type::Custom("dataframe".into()),
                Type::Custom("dataframe".into()),
            )
            .category(Category::Custom("lazyframe".into()))
    }

    fn examples(&'_ self) -> Vec<Example<'_>> {
        vec![
            Example {
                description: "Filter dataframe using an expression",
                example:
                    "[[a b]; [6 2] [4 2] [2 2]] | polars into-df | polars filter ((polars col a) >= 4)",
                result: Some(
                    NuDataFrame::try_from_columns(
                        vec![
                            Column::new(
                                "a".to_string(),
                                vec![Value::test_int(6), Value::test_int(4)],
                            ),
                            Column::new(
                                "b".to_string(),
                                vec![Value::test_int(2), Value::test_int(2)],
                            ),
                        ],
                        None,
                    )
                    .expect("simple df for test should not fail")
                    .into_value(Span::test_data()),
                ),
            },
            Example {
                description: "Filter dataframe for rows where dt is within the last 2 days of the maximum dt value",
                example:
                    "[[dt val]; [2025-04-01 1] [2025-04-02 2] [2025-04-03 3] [2025-04-04 4]] | polars into-df | polars filter ((polars col dt) > ((polars col dt | polars max | $in - 2day)))",
                result: Some(
                NuDataFrame::try_from_columns(
                    vec![
                        Column::new(
                            "dt".to_string(),
                            vec![
                                Value::date(
                                    chrono::DateTime::parse_from_str(
                                        "2025-04-03 00:00:00 +0000",
                                        "%Y-%m-%d %H:%M:%S %z",
                                    )
                                    .expect("date calculation should not fail in test"),
                                    Span::test_data(),
                                ),
                                Value::date(
                                    chrono::DateTime::parse_from_str(
                                        "2025-04-04 00:00:00 +0000",
                                        "%Y-%m-%d %H:%M:%S %z",
                                    )
                                    .expect("date calculation should not fail in test"),
                                    Span::test_data(),
                                ),
                            ]
                        ),
                        Column::new(
                            "val".to_string(),
                            vec![Value::test_int(3), Value::test_int(4)],
                        ),
                    ],
                    None,
                )
                .expect("simple df for test should not fail")
                .into_value(Span::test_data()),
            ),
        },
    ]
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let expr_value: Value = call.req(0)?;
        let filter_expr = NuExpression::try_from_value(plugin, &expr_value)?;
        let pipeline_value = input.into_value(call.head)?;
        let lazy = NuLazyFrame::try_from_value_coerce(plugin, &pipeline_value)?;
        command(plugin, engine, call, lazy, filter_expr).map_err(LabeledError::from)
    }
}

fn command(
    plugin: &PolarsPlugin,
    engine: &EngineInterface,
    call: &EvaluatedCall,
    lazy: NuLazyFrame,
    filter_expr: NuExpression,
) -> Result<PipelineData, ShellError> {
    let lazy = NuLazyFrame::new(
        lazy.from_eager,
        lazy.to_polars().filter(filter_expr.into_polars()),
    );
    lazy.to_pipeline_data(plugin, engine, call.head)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::test_polars_plugin_command;

    #[test]
    fn test_examples() -> Result<(), ShellError> {
        test_polars_plugin_command(&LazyFilter)
    }
}
