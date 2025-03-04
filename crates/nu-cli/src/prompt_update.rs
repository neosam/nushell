use crate::util::report_error;
use crate::NushellPrompt;
use log::info;
use nu_engine::eval_subexpression;
use nu_parser::parse;
use nu_protocol::{
    engine::{EngineState, Stack, StateWorkingSet},
    Config, PipelineData, Span, Value,
};
use reedline::Prompt;

// Name of environment variable where the prompt could be stored
pub(crate) const PROMPT_COMMAND: &str = "PROMPT_COMMAND";
pub(crate) const PROMPT_COMMAND_RIGHT: &str = "PROMPT_COMMAND_RIGHT";
pub(crate) const PROMPT_INDICATOR: &str = "PROMPT_INDICATOR";
pub(crate) const PROMPT_INDICATOR_VI_INSERT: &str = "PROMPT_INDICATOR_VI_INSERT";
pub(crate) const PROMPT_INDICATOR_VI_NORMAL: &str = "PROMPT_INDICATOR_VI_NORMAL";
pub(crate) const PROMPT_MULTILINE_INDICATOR: &str = "PROMPT_MULTILINE_INDICATOR";

fn get_prompt_string(
    prompt: &str,
    config: &Config,
    engine_state: &EngineState,
    stack: &mut Stack,
    is_perf_true: bool,
) -> Option<String> {
    stack
        .get_env_var(engine_state, prompt)
        .and_then(|v| match v {
            Value::Block {
                val: block_id,
                captures,
                ..
            } => {
                let block = engine_state.get_block(block_id);
                let mut stack = stack.captures_to_stack(&captures);
                // Use eval_subexpression to force a redirection of output, so we can use everything in prompt
                let ret_val = eval_subexpression(
                    engine_state,
                    &mut stack,
                    block,
                    PipelineData::new(Span::new(0, 0)), // Don't try this at home, 0 span is ignored
                );
                if is_perf_true {
                    info!(
                        "get_prompt_string (block) {}:{}:{}",
                        file!(),
                        line!(),
                        column!()
                    );
                }

                match ret_val {
                    Ok(ret_val) => Some(ret_val),
                    Err(err) => {
                        let working_set = StateWorkingSet::new(engine_state);
                        report_error(&working_set, &err);
                        None
                    }
                }
            }
            Value::String { val: source, .. } => {
                let mut working_set = StateWorkingSet::new(engine_state);
                let (block, _) = parse(&mut working_set, None, source.as_bytes(), true, &[]);
                // Use eval_subexpression to force a redirection of output, so we can use everything in prompt
                let ret_val = eval_subexpression(
                    engine_state,
                    stack,
                    &block,
                    PipelineData::new(Span::new(0, 0)), // Don't try this at home, 0 span is ignored
                )
                .ok();
                if is_perf_true {
                    info!(
                        "get_prompt_string (string) {}:{}:{}",
                        file!(),
                        line!(),
                        column!()
                    );
                }

                ret_val
            }
            _ => None,
        })
        .and_then(|pipeline_data| {
            let output = pipeline_data.collect_string("", config).ok();

            match output {
                Some(mut x) => {
                    // Just remove the very last newline.
                    if x.ends_with('\n') {
                        x.pop();
                    }

                    if x.ends_with('\r') {
                        x.pop();
                    }
                    Some(x)
                }
                None => None,
            }
        })
}

pub(crate) fn update_prompt<'prompt>(
    config: &Config,
    engine_state: &EngineState,
    stack: &Stack,
    nu_prompt: &'prompt mut NushellPrompt,
    is_perf_true: bool,
) -> &'prompt dyn Prompt {
    let mut stack = stack.clone();

    let left_prompt_string = get_prompt_string(
        PROMPT_COMMAND,
        config,
        engine_state,
        &mut stack,
        is_perf_true,
    );

    let right_prompt_string = get_prompt_string(
        PROMPT_COMMAND_RIGHT,
        config,
        engine_state,
        &mut stack,
        is_perf_true,
    );

    let prompt_indicator_string = get_prompt_string(
        PROMPT_INDICATOR,
        config,
        engine_state,
        &mut stack,
        is_perf_true,
    );

    let prompt_multiline_string = get_prompt_string(
        PROMPT_MULTILINE_INDICATOR,
        config,
        engine_state,
        &mut stack,
        is_perf_true,
    );

    let prompt_vi_insert_string = get_prompt_string(
        PROMPT_INDICATOR_VI_INSERT,
        config,
        engine_state,
        &mut stack,
        is_perf_true,
    );

    let prompt_vi_normal_string = get_prompt_string(
        PROMPT_INDICATOR_VI_NORMAL,
        config,
        engine_state,
        &mut stack,
        is_perf_true,
    );

    // apply the other indicators
    nu_prompt.update_all_prompt_strings(
        left_prompt_string,
        right_prompt_string,
        prompt_indicator_string,
        prompt_multiline_string,
        (prompt_vi_insert_string, prompt_vi_normal_string),
    );

    let ret_val = nu_prompt as &dyn Prompt;
    if is_perf_true {
        info!("update_prompt {}:{}:{}", file!(), line!(), column!());
    }

    ret_val
}
