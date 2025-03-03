use std::{sync::Mutex, time::Duration};

use probe_rs_cli_util::common_options::ProbeOptions;

pub fn run_gdb_server(
    common: ProbeOptions,
    connection_string: Option<&str>,
    reset_halt: bool,
) -> anyhow::Result<()> {
    let session = Mutex::new(common.simple_attach()?);

    if reset_halt {
        session
            .lock()
            .unwrap()
            .core(0)?
            .reset_and_halt(Duration::from_millis(100))?;
    }

    let gdb_connection_string = connection_string.unwrap_or("localhost:1337");
    println!("Firing up GDB stub at {}", gdb_connection_string);

    if let Err(e) = probe_rs_gdb_server::run(Some(gdb_connection_string.to_owned()), &session) {
        eprintln!("During the execution of GDB an error was encountered:");
        eprintln!("{:?}", e);
    }

    Ok(())
}
