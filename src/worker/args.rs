use structopt::StructOpt;

#[derive(StructOpt)]
pub(crate) struct WorkerArgs {
    /// Server address
    #[structopt(name = "ADDRESS", default_value = "localhost")]
    pub address: String,
    /// Server port
    #[structopt(short = "p", long = "port", default_value = "4049")]
    pub port: u16,
    /// Worker name
    #[structopt(short = "n", long = "name")]
    pub name: Option<String>,
}
