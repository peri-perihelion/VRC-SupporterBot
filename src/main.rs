use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

mod settings;
mod discord;
mod github;

fn main() {
    let settings = match settings::load(){
        Ok(s) => s,
        Err(e) => {
            return println!("Failed to load settings: {}", e)
        }
    }; 

    let members = match discord::fetch_usernames(settings.discord){
        Ok(r) => r,
        Err(e) => {
            return println!("Failed to fetch members in roles: {}", e)
        }
    };

    let mut result: Vec<String> = Vec::new();
    for (role, members) in members {
        result.push(format!("{}", &role));
        result.push(format!("{}{}{}", &settings.head, &members.join(&settings.delimiter), &settings.tail));
    }

    match github::push_changes(settings.github, format!("{}{}", &settings.preface, result.join(&settings.joiner))){
        Ok(ok) => ok,
        Err(e) => return println!("Failed to update gist: {}", e)
    };

    if *&settings.close_delay > 0 {
        for n in 0..settings.close_delay{
            print!("\rUpdate complete. Closing in {}.. ", 5 - n);

            std::io::stdout().flush().unwrap();
            sleep(Duration::from_secs(1));
        }
        println!();

        std::process::exit(0);
    }
}