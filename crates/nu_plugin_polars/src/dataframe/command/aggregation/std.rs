use crate::values::{
    cant_convert_err, Column, CustomValueSupport, NuLazyFrame, PolarsPluginObject, PolarsPluginType,
};
use crate::PolarsPlugin;
use crate::{dataframe::values::NuExpression, values::NuDataFrame};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, ShellError, Signature, Span, Type, Value,
};
use polars::df;

pub struct ExprStd;

impl PluginCommand for ExprStd {
    type Plugin = PolarsPlugin;

    fn name(&self) -> &str {
        "polars std"
    }

    fn description(&self) -> &str {
        "Creates a std expression for an aggregation of std value from columns in a dataframe."
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![
                (
                    Type::Custom("expression".into()),
                    Type::Custom("expression".into()),
                ),
                (
                    Type::Custom("dataframe".into()),
                    Type::Custom("dataframe".into()),
                ),
            ])
            .category(Category::Custom("dataframe".into()))
    }

    fn examples(&'_ self) -> Vec<Example<'_>> {
        vec![
            Example {
                description: "Std value from columns in a dataframe",
                example:
                    "[[a b]; [6 2] [4 2] [2 2]] | polars into-df | polars std | polars collect",
                result: Some(
                    NuDataFrame::try_from_columns(
                        vec![
                            Column::new("a".to_string(), vec![Value::test_float(2.0)]),
                            Column::new("b".to_string(), vec![Value::test_float(0.0)]),
                        ],
                        None,
                    )
                    .expect("simple df for test should not fail")
                    .into_value(Span::test_data()),
                ),
            },
            Example {
                description: "Std aggregation for a group-by",
                example: r#"[[a b]; [one 2] [one 2] [two 1] [two 1]]
            | polars into-df
            | polars group-by a
            | polars agg (polars col b | polars std)
            | polars collect
            | polars sort-by a"#,
                result: Some(
                    NuDataFrame::from(
                        df!(
                            "a" => &["one", "two"],
                            "b" => &[0.0f64, 0.0],
                        )
                        .expect("should not fail"),
                    )
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
        let value = input.into_value(call.head)?;
        match PolarsPluginObject::try_from_value(plugin, &value)? {
            PolarsPluginObject::NuDataFrame(df) => command_lazy(plugin, engine, call, df.lazy()),
            PolarsPluginObject::NuLazyFrame(lazy) => command_lazy(plugin, engine, call, lazy),
            PolarsPluginObject::NuExpression(expr) => command_expr(plugin, engine, call, expr),
            _ => Err(cant_convert_err(
                &value,
                &[
                    PolarsPluginType::NuDataFrame,
                    PolarsPluginType::NuLazyFrame,
                    PolarsPluginType::NuExpression,
                ],
            )),
        }
        .map_err(LabeledError::from)
    }
}

fn command_expr(
    plugin: &PolarsPlugin,
    engine: &EngineInterface,
    call: &EvaluatedCall,
    expr: NuExpression,
) -> Result<PipelineData, ShellError> {
    NuExpression::from(expr.into_polars().std(1)).to_pipeline_data(plugin, engine, call.head)
}

fn command_lazy(
    plugin: &PolarsPlugin,
    engine: &EngineInterface,
    call: &EvaluatedCall,
    lazy: NuLazyFrame,
) -> Result<PipelineData, ShellError> {
    let res: NuLazyFrame = lazy.to_polars().std(1).into();

    res.to_pipeline_data(plugin, engine, call.head)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::test_polars_plugin_command;

    #[test]
    fn test_examples() -> Result<(), ShellError> {
        test_polars_plugin_command(&ExprStd)
    }
}
