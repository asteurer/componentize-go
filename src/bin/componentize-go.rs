use anyhow::Result;
use std::env;

fn main() -> Result<()> {
    pretty_env_logger::init_timed();
    componentize_go::command::run(env::args_os())
}
