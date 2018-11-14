extern crate app_dirs;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate failure;
extern crate structopt;
extern crate toml;

use app_dirs::{AppDataType, AppInfo};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
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
    #[structopt(name = "status", about = "current pomodoro status")]
    Status {},
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    count: u8,
    duration: u64,
    rest: u64,
}

impl Config {
    fn cfg_path() -> Result<PathBuf, failure::Error> {
        Ok(app_dirs::app_dir(AppDataType::UserConfig, &APP_INFO, "")?.join("config.toml"))
    }

    fn load() -> Result<Config, failure::Error> {
        let config = match File::open(&Config::cfg_path()?) {
            Ok(mut file) => {
                let mut toml = String::new();
                file.read_to_string(&mut toml)?;
                toml::from_str(&toml)?
            }
            Err(_) => Default::default(),
        };
        Ok(config)
    }
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

#[derive(Debug, Serialize, Deserialize)]
struct State {
    started_at: Option<u64>,
    #[serde(skip)]
    config: Config,
}

impl Default for State {
    fn default() -> State {
        State {
            started_at: None,
            config: Default::default(),
        }
    }
}

impl State {
    fn state_path() -> Result<PathBuf, failure::Error> {
        Ok(app_dirs::app_dir(AppDataType::UserData, &APP_INFO, "")?.join("state.toml"))
    }

    fn load(config: Config) -> Result<State, failure::Error> {
        let state = match File::open(&State::state_path()?) {
            Ok(mut file) => {
                let mut toml = String::new();
                file.read_to_string(&mut toml)?;
                let mut state: State = toml::from_str(&toml)?;
                state.config = config;
                state
            }
            Err(_) => Default::default(),
        };
        Ok(state)
    }

    fn save(&self) -> Result<(), failure::Error> {
        let toml = toml::to_string(&self)?;
        let mut file = File::create(&State::state_path()?)?;
        file.write_all(toml.as_bytes())?;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), failure::Error> {
        self.started_at = None;
        Ok(self.save()?)
    }

    fn display(&self, current_time: u64) -> String {
        if let Some(started_at) = self.started_at {
            let elapsed = current_time - started_at;
            let elapsed_min = elapsed / 60;
            if elapsed_min <= self.config.duration {
                format!("W: {}m", self.config.duration - elapsed_min)
            } else if elapsed_min < (self.config.duration + self.config.rest) {
                format!(
                    "R: {}m",
                    (self.config.duration + self.config.rest) - elapsed_min
                )
            } else {
                format!("READY")
            }
        } else {
            "no pomodoro running".to_string()
        }
    }
}

#[test]
fn test_display_for_state() {
    assert_eq!(
        format!(
            "{}",
            State {
                started_at: None,
                ..Default::default()
            }.display(10)
        ),
        "no pomodoro running"
    );
    assert_eq!(
        format!(
            "{}",
            State {
                started_at: Some(0),
                ..Default::default()
            }.display(600)
        ),
        "W: 15m"
    );
    assert_eq!(
        format!(
            "{}",
            State {
                started_at: Some(0),
                ..Default::default()
            }.display(1600)
        ),
        "R: 4m"
    );
    assert_eq!(
        format!(
            "{}",
            State {
                started_at: Some(0),
                ..Default::default()
            }.display(1801)
        ),
        "READY"
    );
}

fn main() -> Result<(), failure::Error> {
    let opt = Marinara::from_args();
    match opt {
        Marinara::Init { force } => {
            let cfg_path =
                app_dirs::app_dir(AppDataType::UserConfig, &APP_INFO, "")?.join("config.toml");
            if force {
                let cfg: Config = Default::default();
                let toml = toml::to_string(&cfg)?;
                let mut file = File::create(&cfg_path)?;
                file.write_all(toml.as_bytes())?;
                println!("wrote new config to {}", &cfg_path.display());
            }
        }
        Marinara::Start {} => {
            let config = Config::load()?;
            let state = State {
                started_at: Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()),
                config,
            };
            state.save()?;
        }
        Marinara::Stop {} => match State::load(Config::load()?) {
            Ok(mut state) => {
                state.reset()?;
            }
            Err(_) => {
                println!("no pomodoro running");
            }
        },
        Marinara::Status {} => match Config::load() {
            Ok(config) => {
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                match State::load(config) {
                    Ok(state) => {
                        println!("{}", state.display(current_time));
                    }
                    Err(_) => {
                        println!("no pomodoro running");
                    }
                }
            }
            Err(e) => {
                println!("missing config {:?}", e);
            }
        },
    }
    Ok(())
}
