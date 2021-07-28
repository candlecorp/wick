use wapc_guest::HandlerResult;

mod generated;

#[no_mangle]
pub fn wapc_init() {
    generated::Handlers::register_echo(echo);
}

fn echo(input: String) -> HandlerResult<String> {
    Ok(input)
}
