use crate::{
    values::{
        cant_convert_err, CustomValueSupport, NuExpression, NuLazyFrame, PolarsPluginObject,
        PolarsPluginType,
    },
    PolarsPlugin,
};

use super::super::super::values::NuDataFrame;

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, ShellError, Signature, Span, Type,
};
use polars::{
    prelude::{col, DatetimeMethods, IntoSeries, NamedFrom},
    series::Series,
};

#[derive(Clone)]
pub struct GetHour;

impl PluginCommand for GetHour {
    type Plugin = PolarsPlugin;

    fn name(&self) -> &str {
        "polars get-hour"
    }

    fn description(&self) -> &str {
        "Gets hour from datetime."
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![
                (
                    Type::Custom("dataframe".into()),
                    Type::Custom("dataframe".into()),
                ),
                (
                    Type::Custom("expression".into()),
                    Type::Custom("expression".into()),
                ),
            ])
            .category(Category::Custom("dataframe".into()))
    }

    fn examples(&'_ self) -> Vec<Example<'_>> {
        vec![
            Example {
                description: "Returns hour from a date",
                example: r#"let dt = ('2020-08-04T16:39:18+00:00' | into datetime --timezone 'UTC');
    let df = ([$dt $dt] | polars into-df);
    $df | polars get-hour"#,
                result: Some(
                    NuDataFrame::try_from_series(
                        Series::new("0".into(), &[16i8, 16]),
                        Span::test_data(),
                    )
                    .expect("simple df for test should not fail")
                    .into_value(Span::test_data()),
                ),
            },
            Example {
                description: "Returns hour from a date in a lazyframe",
                example: r#"let dt = ('2020-08-04T16:39:18+00:00' | into datetime --timezone 'UTC');
    let df = ([$dt $dt] | polars into-lazy);
    $df | polars get-hour"#,
                result: Some(
                    NuDataFrame::try_from_series(
                        Series::new("0".into(), &[16i8, 16]),
                        Span::test_data(),
                    )
                    .expect("simple df for test should not fail")
                    .into_value(Span::test_data()),
                ),
            },
            Example {
                description: "Returns hour from a date in an expression",
                example: r#"let dt = ('2020-08-04T16:39:18+00:00' | into datetime --timezone 'UTC');
    let df = ([$dt $dt] | polars into-df);
    $df | polars select (polars col 0 | polars get-hour)"#,
                result: Some(
                    NuDataFrame::try_from_series(
                        Series::new("0".into(), &[16i8, 16]),
                        Span::test_data(),
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
        command(plugin, engine, call, input).map_err(LabeledError::from)
    }
}

fn command(
    plugin: &PolarsPlugin,
    engine: &EngineInterface,
    call: &EvaluatedCall,
    input: PipelineData,
) -> Result<PipelineData, ShellError> {
    let value = input.into_value(call.head)?;

    match PolarsPluginObject::try_from_value(plugin, &value)? {
        PolarsPluginObject::NuLazyFrame(lazy) => command_lazy(plugin, engine, call, lazy),
        PolarsPluginObject::NuDataFrame(df) => command_eager(plugin, engine, call, df),
        PolarsPluginObject::NuExpression(expr) => {
            let res: NuExpression = expr.into_polars().dt().hour().into();
            res.to_pipeline_data(plugin, engine, call.head)
        }
        _ => Err(cant_convert_err(
            &value,
            &[
                PolarsPluginType::NuDataFrame,
                PolarsPluginType::NuLazyFrame,
                PolarsPluginType::NuExpression,
            ],
        )),
    }
}

fn command_lazy(
    plugin: &PolarsPlugin,
    engine: &EngineInterface,
    call: &EvaluatedCall,
    lazy: NuLazyFrame,
) -> Result<PipelineData, ShellError> {
    NuLazyFrame::new(false, lazy.to_polars().select([col("*").dt().hour()]))
        .to_pipeline_data(plugin, engine, call.head)
}

fn command_eager(
    plugin: &PolarsPlugin,
    engine: &EngineInterface,
    call: &EvaluatedCall,
    df: NuDataFrame,
) -> Result<PipelineData, ShellError> {
    let series = df.as_series(call.head)?;

    let casted = series.datetime().map_err(|e| ShellError::GenericError {
        error: "Error casting to datetime type".into(),
        msg: e.to_string(),
        span: Some(call.head),
        help: None,
        inner: vec![],
    })?;

    let res = casted.hour().into_series();

    let df = NuDataFrame::try_from_series_vec(vec![res], call.head)?;
    df.to_pipeline_data(plugin, engine, call.head)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::test_polars_plugin_command_with_decls;
    use nu_command::IntoDatetime;

    #[test]
    fn test_examples() -> Result<(), ShellError> {
        test_polars_plugin_command_with_decls(&GetHour, vec![Box::new(IntoDatetime)])
    }
}
