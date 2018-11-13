extern crate app_dirs;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate failure;
extern crate structopt;
extern crate toml;

use app_dirs::{AppDataType, AppInfo};
use std::fs::File;
use std::io::Write;
use structopt::StructOpt;

const APP_INFO: AppInfo = AppInfo {
    name: "marinara",
    author: "Douglas Campos <qmx@qmx.me>",
};

#[derive(Debug, StructOpt)]
#[structopt(name = "marinara", about = "pomodoro timer")]
enum Marinara {
    #[structopt(name = "init", about = "initialize configuration")]
    Init {
        #[structopt(short = "f", long = "force")]
        /// Create a new config file with defaults, unconditionally
        force: bool,
    },
    #[structopt(name = "start", about = "start a new pomodoro")]
    Start {},
    #[structopt(name = "stop", about = "stop current pomodoro")]
    Stop {},
    #[structopt(name = "pause", about = "pause current pomodoro")]
    Pause {},
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    count: u8,
    duration: u8,
    rest: u8,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            count: 8,
            duration: 25,
            rest: 5,
        }
    }
}

fn main() -> Result<(), failure::Error> {
    let opt = Marinara::from_args();
    match opt {
        Marinara::Init { force } => {
            if force {
                let cfg: Config = Default::default();
                let toml = toml::to_string(&cfg)?;
                let cfg_path =
                    app_dirs::app_dir(AppDataType::UserConfig, &APP_INFO, "")?.join("config.toml");
                let mut file = File::create(&cfg_path)?;
                file.write_all(toml.as_bytes())?;
                println!("wrote new config to {}", &cfg_path.display());
            }
        }
        Marinara::Start {} => {}
        Marinara::Stop {} => {}
        Marinara::Pause {} => {}
    };
    Ok(())
}
