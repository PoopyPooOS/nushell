use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, ClearType},
};
use itertools::Itertools;
use nu_engine::command_prelude::*;
use nu_protocol::shell_error::io::IoError;

use std::{io::Write, time::Duration};

#[derive(Clone)]
pub struct Input;

impl Command for Input {
    fn name(&self) -> &str {
        "input"
    }

    fn description(&self) -> &str {
        "Get input from the user."
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["prompt", "interactive"]
    }

    fn signature(&self) -> Signature {
        Signature::build("input")
            .input_output_types(vec![(Type::Nothing, Type::Any)])
            .allow_variants_without_examples(true)
            .optional("prompt", SyntaxShape::String, "Prompt to show the user.")
            .named(
                "bytes-until-any",
                SyntaxShape::String,
                "read bytes (not text) until any of the given stop bytes is seen",
                Some('u'),
            )
            .named(
                "numchar",
                SyntaxShape::Int,
                "number of characters to read; suppresses output",
                Some('n'),
            )
            .named(
                "default",
                SyntaxShape::String,
                "default value if no input is provided",
                Some('d'),
            )
            .switch("suppress-output", "don't print keystroke values", Some('s'))
            .category(Category::Platform)
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let prompt: Option<String> = call.opt(engine_state, stack, 0)?;
        let bytes_until: Option<String> = call.get_flag(engine_state, stack, "bytes-until-any")?;
        let suppress_output = call.has_flag(engine_state, stack, "suppress-output")?;
        let numchar: Option<Spanned<i64>> = call.get_flag(engine_state, stack, "numchar")?;
        let numchar: Spanned<i64> = numchar.unwrap_or(Spanned {
            item: i64::MAX,
            span: call.head,
        });

        let from_io_error = IoError::factory(call.head, None);

        if numchar.item < 1 {
            return Err(ShellError::UnsupportedInput {
                msg: "Number of characters to read has to be positive".to_string(),
                input: "value originated from here".to_string(),
                msg_span: call.head,
                input_span: numchar.span,
            });
        }

        let default_val: Option<String> = call.get_flag(engine_state, stack, "default")?;
        if let Some(prompt) = &prompt {
            match &default_val {
                None => print!("{prompt}"),
                Some(val) => print!("{prompt} (default: {val})"),
            }
            let _ = std::io::stdout().flush();
        }

        let mut buf = String::new();

        crossterm::terminal::enable_raw_mode().map_err(&from_io_error)?;
        // clear terminal events
        while crossterm::event::poll(Duration::from_secs(0)).map_err(&from_io_error)? {
            // If there's an event, read it to remove it from the queue
            let _ = crossterm::event::read().map_err(&from_io_error)?;
        }

        loop {
            if i64::try_from(buf.len()).unwrap_or(0) >= numchar.item {
                break;
            }
            match crossterm::event::read() {
                Ok(Event::Key(k)) => match k.kind {
                    KeyEventKind::Press | KeyEventKind::Repeat => {
                        match k.code {
                            // TODO: maintain keycode parity with existing command
                            KeyCode::Char(c) => {
                                if k.modifiers == KeyModifiers::ALT
                                    || k.modifiers == KeyModifiers::CONTROL
                                {
                                    if k.modifiers == KeyModifiers::CONTROL && c == 'c' {
                                        crossterm::terminal::disable_raw_mode()
                                            .map_err(&from_io_error)?;
                                        return Err(IoError::new(
                                            std::io::ErrorKind::Interrupted,
                                            call.head,
                                            None,
                                        )
                                        .into());
                                    }
                                    continue;
                                }

                                if let Some(bytes_until) = bytes_until.as_ref() {
                                    if bytes_until.bytes().contains(&(c as u8)) {
                                        break;
                                    }
                                }
                                buf.push(c);
                            }
                            KeyCode::Backspace => {
                                let _ = buf.pop();
                            }
                            KeyCode::Enter => {
                                break;
                            }
                            _ => continue,
                        }
                    }
                    _ => continue,
                },
                Ok(_) => continue,
                Err(event_error) => {
                    crossterm::terminal::disable_raw_mode().map_err(&from_io_error)?;
                    return Err(from_io_error(event_error).into());
                }
            }
            if !suppress_output {
                // clear the current line and print the current buffer
                execute!(
                    std::io::stdout(),
                    terminal::Clear(ClearType::CurrentLine),
                    cursor::MoveToColumn(0),
                )
                .map_err(|err| IoError::new(err.kind(), call.head, None))?;
                if let Some(prompt) = &prompt {
                    execute!(std::io::stdout(), Print(prompt.to_string()))
                        .map_err(&from_io_error)?;
                }
                execute!(std::io::stdout(), Print(buf.to_string())).map_err(&from_io_error)?;
            }
        }
        crossterm::terminal::disable_raw_mode().map_err(&from_io_error)?;
        if !suppress_output {
            std::io::stdout().write_all(b"\n").map_err(&from_io_error)?;
        }
        match default_val {
            Some(val) if buf.is_empty() => Ok(Value::string(val, call.head).into_pipeline_data()),
            _ => Ok(Value::string(buf, call.head).into_pipeline_data()),
        }
    }

    fn examples(&'_ self) -> Vec<Example<'_>> {
        vec![
            Example {
                description: "Get input from the user, and assign to a variable",
                example: "let user_input = (input)",
                result: None,
            },
            Example {
                description: "Get two characters from the user, and assign to a variable",
                example: "let user_input = (input --numchar 2)",
                result: None,
            },
            Example {
                description: "Get input from the user with default value, and assign to a variable",
                example: "let user_input = (input --default 10)",
                result: None,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::Input;

    #[test]
    fn examples_work_as_expected() {
        use crate::test_examples;
        test_examples(Input {})
    }
}
