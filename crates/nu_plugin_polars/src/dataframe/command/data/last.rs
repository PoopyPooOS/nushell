use crate::{
    values::{Column, CustomValueSupport, NuLazyFrame, PolarsPluginObject},
    PolarsPlugin,
};

use crate::values::{utils::DEFAULT_ROWS, NuDataFrame, NuExpression};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, ShellError, Signature, Span, SyntaxShape, Type,
    Value,
};

#[derive(Clone)]
pub struct LastDF;

impl PluginCommand for LastDF {
    type Plugin = PolarsPlugin;

    fn name(&self) -> &str {
        "polars last"
    }

    fn description(&self) -> &str {
        "Creates new dataframe with tail rows or creates a last expression."
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .optional("rows", SyntaxShape::Int, "Number of rows for tail")
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
                description: "Create new dataframe with last rows",
                example: "[[a b]; [1 2] [3 4]] | polars into-df | polars last 1",
                result: Some(
                    NuDataFrame::try_from_columns(
                        vec![
                            Column::new("a".to_string(), vec![Value::test_int(3)]),
                            Column::new("b".to_string(), vec![Value::test_int(4)]),
                        ],
                        None,
                    )
                    .expect("simple df for test should not fail")
                    .into_value(Span::test_data()),
                ),
            },
            Example {
                description: "Creates a last expression from a column",
                example: "polars col a | polars last",
                result: None,
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
            PolarsPluginObject::NuDataFrame(df) => {
                command_eager(plugin, engine, call, df).map_err(|e| e.into())
            }
            PolarsPluginObject::NuLazyFrame(lazy) => {
                command_lazy(plugin, engine, call, lazy).map_err(|e| e.into())
            }
            _ => {
                let expr = NuExpression::try_from_value(plugin, &value)?;
                let expr: NuExpression = expr.into_polars().last().into();

                expr.to_pipeline_data(plugin, engine, call.head)
                    .map_err(LabeledError::from)
            }
        }
    }
}

fn command_eager(
    plugin: &PolarsPlugin,
    engine: &EngineInterface,
    call: &EvaluatedCall,
    df: NuDataFrame,
) -> Result<PipelineData, ShellError> {
    let rows: Option<usize> = call.opt(0)?;
    let rows = rows.unwrap_or(DEFAULT_ROWS);

    let res = df.as_ref().tail(Some(rows));
    let res = NuDataFrame::new(false, res);
    res.to_pipeline_data(plugin, engine, call.head)
}

fn command_lazy(
    plugin: &PolarsPlugin,
    engine: &EngineInterface,
    call: &EvaluatedCall,
    lazy: NuLazyFrame,
) -> Result<PipelineData, ShellError> {
    let rows: Option<u64> = call.opt(0)?;
    let rows = rows.unwrap_or(DEFAULT_ROWS as u64);

    let res: NuLazyFrame = lazy.to_polars().tail(rows).into();

    res.to_pipeline_data(plugin, engine, call.head)
}

#[cfg(test)]
mod test {
    use crate::test::test_polars_plugin_command;

    use super::*;

    #[test]
    fn test_examples() -> Result<(), ShellError> {
        test_polars_plugin_command(&LastDF)
    }
}
