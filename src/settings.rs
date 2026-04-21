use std::{fs::{self, File}, io::{Read, Write}};
use serde::{Serialize, Deserialize};
use indexmap::IndexMap;

#[derive(Deserialize, Serialize)]
pub struct Config {
    /// Text at the beginning of the full supporter list.
    pub preface: String,
    /// Seperator between each supporter username.
    pub delimiter: String,
    /// Seperator between blocks of supporter roles and their respective usernames.
    pub joiner: String,
    /// Text at the beginning of each block of supporter usernames.
    pub head: String,
    /// Text at the end of each block of supporter usernames.
    pub tail: String,

    /// Countdown before exiting the program.
    pub close_delay: i32,

    pub discord: Discord,
    pub github: Github,
}
#[derive(Deserialize, Serialize)]
pub struct Discord {
    pub token: String,
    pub server_id: u64,
    pub roles: IndexMap<String, Vec<u64>>
}
#[derive(Deserialize, Serialize)]
pub struct Github {
    pub token: String,
    pub gist_id: String,
    pub gist_name: String,
}

// settings template
const TEMPLATE: &str = 
r#"# Report bugs at https://github.com/peri-perihelion/VRC-SupporterBot

# text at the beginning of the file, the title
preface = ""

# the seperator between each supporter username
delimiter = "   "
# the seperator between tier titles and supporter usernames
joiner = "\n"

# text at the start of each block of supporter names
head = ""
# text at the end of each block of supporter names
tail = "</color>\n"

# time (in seconds) until the program closes once its completed the upload
close_delay = 5

[discord]
# discord bot token from: https://discord.com/developers/applications
token = ""
server_id = 0
# each of the tiers, and their role ids, in the order you want them to appear
#   the kofi and patreon discord bots work best when they have their own separate roles, so this allows you to do that
[discord.roles]
"<color=#2596be><b>Example tier, with two role ids</b>" = [12345, 67890]
"<color=#df783e><b>A second example tier, but with only one role id this time</b>" = [12345]

[github]
# github access token from: https://github.com/settings/personal-access-tokens
token = ""
# this two will be filled automatically, dont fill them out unless you want to specify a gist to edit
gist_id = ""
gist_name = ""
"#;

/// Updates the selected setting
pub fn update_setting(setting: &str, value: &str) -> Result<(), String> {
    // read the contents
    let mut raw = match fs::read_to_string("settings.toml"){
        Ok(r) => r,
        Err(e) => return Err(e.to_string())
    };

    // find the index where the setting begins
    let start = match raw.find(setting) {
        Some(i) => i,
        None => return Err(format!("Settings file does not contain {}", setting))
    };

    // find the index where the setting ends
    let end = match raw[start..].find("\n") {
        Some(s) => s + start,
        None => raw.len()
    };

    // set the setting
    raw.replace_range(start..end, &format!("{} = \"{}\"", setting, value));

    // open the settings
    let mut file = match File::create("settings.toml") {
        Ok(f) => f,
        Err(e) => return Err(e.to_string())
    };

    // write the updated settings
    match file.write_all(raw.as_bytes()){
        Ok(_r) => return Ok(()),
        Err(e) => return Err(e.to_string())
    }
}

/// Writes an empty settings file based on the template.
fn write_template() -> Result<File, String> {
    // open a new file
    let mut file = match File::create("settings.toml") {
        Ok(f) => f,
        Err(e) => return Err(e.to_string())
    };

    // write the template
    match file.write_all(TEMPLATE.as_bytes()){
        Ok(_r) => return Ok(file),
        Err(e) => return Err(e.to_string())
    }
}

/// Attempts to load a settings file. Will create a new blank one if there is none present.
pub fn load() -> Result<Config, String>{
    // open the settings
    let mut file = match File::open("settings.toml") {
        Ok(f) => f,
        Err(e) => match e.kind() {
            // settings dont exist, create them and tell the user to restart
            std::io::ErrorKind::NotFound => match write_template() {
                Ok(_s) => return Err("Created settings file, please fill it out and restart.".to_string()),
                Err(e) => return Err(e)
            }
            // unknown issue
            _ => return Err(e.kind().to_string())
        }
    };

    // read the file contents
    let mut raw_toml = String::new();
    match file.read_to_string(&mut raw_toml){
        Ok(s) => s,
        Err(e) => return Err(e.to_string())
    };

    // convert to toml
    match toml::from_str(&mut raw_toml){
        Ok(s) => return Ok(s),
        Err(e) =>  return Err(e.to_string())
    };
}