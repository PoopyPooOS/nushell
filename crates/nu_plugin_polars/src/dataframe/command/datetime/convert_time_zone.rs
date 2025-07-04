use crate::values::{Column, NuDataFrame, NuSchema};
use crate::{
    dataframe::values::NuExpression,
    values::{cant_convert_err, CustomValueSupport, PolarsPluginObject, PolarsPluginType},
    PolarsPlugin,
};

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, Span, SyntaxShape, Type, Value,
};

use chrono::DateTime;
use polars::prelude::*;

#[derive(Clone)]
pub struct ConvertTimeZone;

impl PluginCommand for ConvertTimeZone {
    type Plugin = PolarsPlugin;

    fn name(&self) -> &str {
        "polars convert-time-zone"
    }

    fn description(&self) -> &str {
        "Convert datetime to target timezone."
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(
                Type::Custom("expression".into()),
                Type::Custom("expression".into()),
            )])
            .required(
                "time_zone",
                SyntaxShape::String,
                "Timezone for the Datetime Series. Pass `null` to unset time zone.",
            )
            .category(Category::Custom("dataframe".into()))
    }

    fn examples(&'_ self) -> Vec<Example<'_>> {
        vec![
            Example {
                description: "Convert timezone for timezone-aware datetime",
                example: r#"["2025-04-10 09:30:00 -0400" "2025-04-10 10:30:00 -0400"] | polars into-df
                    | polars as-datetime "%Y-%m-%d %H:%M:%S %z"
                    | polars select (polars col datetime | polars convert-time-zone "Europe/Lisbon")"#,
                result: Some(
                    NuDataFrame::try_from_columns(
                        vec![Column::new(
                            "datetime".to_string(),
                            vec![
                                Value::date(
                                    DateTime::parse_from_str(
                                        "2025-04-10 14:30:00 +0100",
                                        "%Y-%m-%d %H:%M:%S %z",
                                    )
                                    .expect("date calculation should not fail in test"),
                                    Span::test_data(),
                                ),
                                Value::date(
                                    DateTime::parse_from_str(
                                        "2025-04-10 15:30:00 +0100",
                                        "%Y-%m-%d %H:%M:%S %z",
                                    )
                                    .expect("date calculation should not fail in test"),
                                    Span::test_data(),
                                ),
                            ],
                        )],
                        Some(NuSchema::new(Arc::new(Schema::from_iter(vec![
                            Field::new(
                                "datetime".into(),
                                DataType::Datetime(
                                    TimeUnit::Nanoseconds,
                                    Some(PlSmallStr::from_static("Europe/Lisbon")),
                                ),
                            ),
                        ])))),
                    )
                    .expect("simple df for test should not fail")
                    .into_value(Span::test_data()),
                ),
            },
            Example {
                description: "Timezone conversions for timezone-naive datetime will assume the original timezone is UTC",
                example: r#"["2025-04-10 09:30:00" "2025-04-10 10:30:00"] | polars into-df
                    | polars as-datetime "%Y-%m-%d %H:%M:%S" --naive
                    | polars select (polars col datetime | polars convert-time-zone "America/New_York")"#,
                result: Some(
                    NuDataFrame::try_from_columns(
                        vec![Column::new(
                            "datetime".to_string(),
                            vec![
                                Value::date(
                                    DateTime::parse_from_str(
                                        "2025-04-10 05:30:00 -0400",
                                        "%Y-%m-%d %H:%M:%S %z",
                                    )
                                    .expect("date calculation should not fail in test"),
                                    Span::test_data(),
                                ),
                                Value::date(
                                    DateTime::parse_from_str(
                                        "2025-04-10 06:30:00 -0400",
                                        "%Y-%m-%d %H:%M:%S %z",
                                    )
                                    .expect("date calculation should not fail in test"),
                                    Span::test_data(),
                                ),
                            ],
                        )],
                        Some(NuSchema::new(Arc::new(Schema::from_iter(vec![
                            Field::new(
                                "datetime".into(),
                                DataType::Datetime(
                                    TimeUnit::Nanoseconds,
                                    Some(PlSmallStr::from_static("America/New_York")),
                                ),
                            ),
                        ])))),
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
        let value = input.into_value(call.head)?;

        match PolarsPluginObject::try_from_value(plugin, &value)? {
            PolarsPluginObject::NuExpression(expr) => {
                let time_zone: String = call.req(0)?;
                let expr: NuExpression = expr
                    .into_polars()
                    .dt()
                    .convert_time_zone(PlSmallStr::from_str(&time_zone))
                    .into();
                expr.to_pipeline_data(plugin, engine, call.head)
            }
            _ => Err(cant_convert_err(&value, &[PolarsPluginType::NuExpression])),
        }
        .map_err(LabeledError::from)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::test::test_polars_plugin_command;
    use nu_protocol::ShellError;

    #[test]
    fn test_examples() -> Result<(), ShellError> {
        test_polars_plugin_command(&ConvertTimeZone)
    }
}
