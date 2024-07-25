pub fn get_clap_command() -> clap::Command {
    clap::Command::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            clap::Arg::new("fullscreen")
                .short('f')
                .long("fullscreen")
                .help("Start simp in fullscreen")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("generate-man")
                .long("generate-man")
                .help("Generates manual page for quickshot")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(clap::Arg::new("file").help("Load this file").index(1))
}
