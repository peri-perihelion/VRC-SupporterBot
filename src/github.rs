use octocrab::Octocrab;
use tokio::runtime;

use crate::settings;

/// Attempts to push the new contents to the specified gist.
pub fn push_changes(config: settings::Github, contents: String) -> Result<String, String>{
    println!("Starting Github client..");

    let runtime = match runtime::Builder::new_current_thread()
        .enable_all()
        .build(){
            Ok(rt) => rt,
            Err(e) => return Err(e.to_string())
        };
        
    runtime.block_on(async{
        let client = match Octocrab::builder().personal_token(&*config.token).build(){
            Ok(o) => o,
            Err(e) => return Err(e.to_string())
        };

        // if we didnt specify a gist id, then create a new one
        if config.gist_id == "" {
            println!("Creating new Gist..");
            // use a default filename if one wasnt specified
            let filename = 
                if config.gist_name.is_empty() { 
                    "supporters.txt" 
                }
                else { 
                    &config.gist_name 
                };
            match create_gist(client, filename, &contents).await {
                Ok(id) => {
                    // update our saved id
                    match settings::update_setting("gist_id", &id) {
                        Ok(_) => {
                            // update our saved name
                            match settings::update_setting("gist_name", filename) {
                                Ok(_) => return Ok("Successfully updated".to_string()),
                                Err(e) => return Err(e)
                            }
                        },
                        Err(e) => return Err(e)
                    }
                }
                Err(e) => return Err(e.to_string())
            }
        }
        else { // otherwise, use the specified one
            println!("Updating Gist..");
            match client
                .gists()
                .update(&config.gist_id)
                .file(&config.gist_name)
                .with_content(contents)
                .send()
            .await {
                Ok(_g) => return Ok("Successfully updated".to_string()),
                Err(e) => return Err(e.to_string())
            };
        }
    })
}

/// Creates a gist under the name "supporters.txt", and returns the Id if it succeeds
async fn create_gist(client: Octocrab, filename: &str, content: &String) -> Result<String, String> {
    match client
        .gists()
        .create()
        .file(filename, content)
        .description("A list of my supporters, automated by https://github.com/peri-perihelion/vrc-supporterbot")
        .public(false)
        .send()
        .await {
            Ok(g) => Ok(g.id),
            Err(e) => return Err(e.to_string())
        }
}