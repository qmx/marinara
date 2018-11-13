extern crate structopt;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "marinara", about = "pomodoro timer")]
enum Marinara {
    #[structopt(name = "init", about = "initialize configuration")]
    Init {
        #[structopt(short = "f", long = "force")]
        /// Create a new config file with defaults, unconditionally
        force:bool
    },
    #[structopt(name = "start", about = "start a new pomodoro")]
    Start {},
    #[structopt(name = "stop", about = "stop current pomodoro")]
    Stop {},
    #[structopt(name = "pause", about = "pause current pomodoro")]
    Pause {},
}

fn main() {
    let opt = Marinara::from_args();
    println!("{:?}", opt);
}
