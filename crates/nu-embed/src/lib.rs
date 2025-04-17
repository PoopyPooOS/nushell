use nu_cli::{eval_source, gather_parent_env_vars};
use nu_cmd_lang::create_default_context;
use nu_command::add_shell_command_context;
pub use nu_protocol::{PipelineData, Value, engine::Command};
use nu_protocol::{Stack, StateWorkingSet};
use std::{collections::HashMap, env::current_dir, path::PathBuf};

pub struct Engine {
    commands: Vec<Box<dyn Command>>,
    env_vars: HashMap<String, Value>,

    input: PipelineData,
    allow_return: bool,
    name: String,

    source: String,
}

impl Engine {
    pub fn new(source: impl Into<String>) -> Self {
        Self::new_with_name(source, "script")
    }

    pub fn new_with_name(source: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            commands: Vec::new(),
            env_vars: HashMap::new(),

            input: PipelineData::Empty,
            allow_return: false,
            name: name.into(),

            source: source.into(),
        }
    }

    /// Add extra commands to the engine state.
    pub fn add_command(&mut self, command: impl Command + 'static) {
        self.commands.push(Box::new(command));
    }

    /// Add extra environment variables to the engine state.
    pub fn add_env_var(&mut self, name: impl Into<String>, value: Value) {
        self.env_vars.insert(name.into(), value);
    }

    /// Set the pipeline input data.
    pub fn set_input(&mut self, input: PipelineData) {
        self.input = input;
    }

    /// Set the script name.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn set_allow_return(&mut self, allow_return: bool) {
        self.allow_return = allow_return;
    }

    pub fn eval(self) {
        let mut engine_state = add_shell_command_context(create_default_context());
        let mut stack = Stack::new();

        engine_state.is_interactive = false;
        gather_parent_env_vars(
            &mut engine_state,
            &current_dir().unwrap_or(PathBuf::from("/")),
        );

        engine_state.generate_nu_constant();

        let delta = {
            let mut working_set = StateWorkingSet::new(&engine_state);

            for command in self.commands {
                working_set.add_decl(command);
            }

            working_set.render()
        };

        if let Err(err) = engine_state.merge_delta(delta) {
            eprintln!("Error adding extra commands to the engine: {err:?}");
        }

        for env_var in self.env_vars {
            stack.add_env_var(env_var.0, env_var.1);
        }

        eval_source(
            &mut engine_state,
            &mut stack,
            self.source.as_bytes(),
            &self.name,
            self.input,
            self.allow_return,
        );
    }
}
