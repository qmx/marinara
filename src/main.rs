use app_dirs;
use failure;
use structopt;
use toml;

use app_dirs::{AppDataType, AppInfo};
use chrono::Duration;
use serde::{Deserialize, Serialize};
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
    #[structopt(name = "start", about = "start a new pomodoro")]
    Start {},
    #[structopt(name = "stop", about = "stop current pomodoro")]
    Stop {},
    #[structopt(name = "status", about = "current pomodoro status")]
    Status {},
}

#[derive(Debug)]
struct Config {
    duration: Duration,
    rest: Duration,
}

impl Config {
    fn total(&self) -> Duration {
        self.duration + self.rest
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            duration: Duration::minutes(25),
            rest: Duration::minutes(5),
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

    fn pomodoro(&self, current_time: u64) -> Option<Pomodoro> {
        if let Some(started_at) = self.started_at {
            let elapsed: i64 = (current_time - started_at) as i64;
            if elapsed <= self.config.duration.num_seconds() {
                Some(Pomodoro::Work {
                    remaining_time: self.config.duration - Duration::seconds(elapsed),
                })
            } else if elapsed < self.config.total().num_seconds() {
                Some(Pomodoro::Rest {
                    remaining_time: self.config.total() - Duration::seconds(elapsed),
                })
            } else {
                Some(Pomodoro::Done)
            }
        } else {
            None
        }
    }
}

trait Display {
    fn display(self) -> String;
}

impl Display for Option<Pomodoro> {
    fn display(self) -> String {
        match self {
            Some(pomodoro) => pomodoro.display(),
            None => ">----".to_string(),
        }
    }
}

#[derive(Debug)]
enum Pomodoro {
    Work { remaining_time: Duration },
    Rest { remaining_time: Duration },
    Done,
}

impl Pomodoro {
    fn prefix(self) -> &'static str {
        match self {
            Pomodoro::Work { .. } => "W",
            Pomodoro::Rest { .. } => "R",
            Pomodoro::Done => ">",
        }
    }

    fn display(self) -> String {
        match self {
            Pomodoro::Work { remaining_time } | Pomodoro::Rest { remaining_time } => {
                if remaining_time.num_minutes() > 0 {
                    format!("{}:{:2}m", self.prefix(), remaining_time.num_minutes())
                } else {
                    format!("{}:{:2}s", self.prefix(), remaining_time.num_seconds())
                }
            }
            Pomodoro::Done => ">DONE".to_string(),
        }
    }
}

#[test]
fn test_pomodoro_display() {
    assert_eq!(
        Pomodoro::Work {
            remaining_time: Duration::minutes(15)
        }
        .display(),
        "W:15m"
    );
    assert_eq!(
        Pomodoro::Work {
            remaining_time: Duration::minutes(3)
        }
        .display(),
        "W: 3m"
    );
    assert_eq!(
        Pomodoro::Work {
            remaining_time: Duration::seconds(45)
        }
        .display(),
        "W:45s"
    );
    assert_eq!(
        Pomodoro::Rest {
            remaining_time: Duration::minutes(3)
        }
        .display(),
        "R: 3m"
    );
    assert_eq!(Pomodoro::Done {}.display(), ">DONE");
}

fn main() -> Result<(), failure::Error> {
    let opt = Marinara::from_args();
    match opt {
        Marinara::Start {} => {
            let config: Config = Default::default();
            let state = State {
                started_at: Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()),
                config,
            };
            state.save()?;
        }
        Marinara::Stop {} => {
            State::load(Default::default())?.reset()?;
        }
        Marinara::Status {} => {
            let state = State::load(Default::default())?;
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            println!("{}", state.pomodoro(current_time).display());
        }
    };
    Ok(())
}
