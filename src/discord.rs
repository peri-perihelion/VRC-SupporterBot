use std::sync::Arc;

use indexmap::IndexMap;
use serenity::all::{GuildId, RoleId, ShardManager};
use serenity::async_trait;
use serenity::prelude::*;
use tokio::runtime;

use crate::settings;

/// Stores the discord settings, for specifying what roles and what sever it should interact with.
struct Handler {
    pub settings: settings::Discord
}
/// Stores the Shard Manager, allowing the bot to shut itself down once finished.
struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}
/// Stores the role name as the key, and a vector of the users within it as the value. 
struct DesiredMembers;
impl TypeMapKey for DesiredMembers {
    type Value = IndexMap<String, Vec<String>>;
}

#[async_trait]
// discord bot events are sent here
impl EventHandler for Handler {
    // runs once the bot has cached all the data for each of the guilds its present in
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>){
        println!("Fetching member list..");
        let members = match GuildId::new(self.settings.server_id).members(&ctx.http, None, None).await {
            Ok(m) => m,
            Err(e) => {
                // errors within the event handler have no way to propogate them up to the discord client itself
                // this is by design, and normally a good thing! but in our case it means we have to panic instead of cleanly returning errors
                panic!("Failed to fetch member list: {}", e)
            }
        };

        // for every role (tier of patreon/kofi), fetch the the users that have at least one of the id's for it
        for (role, ids) in self.settings.roles.iter() {
            // turn our raw id numbers into the RoleId object
            let desired_roles: Vec<RoleId> = ids.iter().map(|r| RoleId::new(*r)).collect();

            // get the usernames of anyone who shares at least one of the RoleId's with our dseired ones
            let mut members_in_role: Vec<String> = members
                .iter().
                filter(|m| shared_elements(&m.roles, &desired_roles))
                .map(|m| format!("{}", &m.user.name))
                .collect();

            // sort while ignoring special characters. this works because a username cant be only special characters
            members_in_role
                .sort_by(|a, b| 
                    strip_special_characters(a.to_string())
                    .cmp(
                        &strip_special_characters(b.to_string())));
            
            // save it to data for later
            insert_members(&ctx, role.to_string(), members_in_role).await
        }

        let data = ctx.data.read().await;
        let shard_manager = match data.get::<ShardManagerContainer>() {
            Some(sm) => sm.clone(),
            None => panic!("Failed to find shard manager. This should never happen, please report this!")
        };
        shard_manager.shutdown_all().await;
    }
}

/// Starts a discord bot, and fetches all of the usernames for each role in the tier
pub fn fetch_usernames(config: settings::Discord) -> Result<IndexMap<String, Vec<String>>, String>{
    let runtime = match runtime::Builder::new_current_thread()
        .enable_all()
        .build(){
            Ok(rt) => rt,
            Err(e) => return Err(e.to_string())
        };

    runtime.block_on(async {
        let discord_intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MEMBERS;
        let handler = Handler{
            settings: config
        };

        let mut discord_client = match Client::builder(&handler.settings.token, discord_intents)
            .event_handler(handler)
            .await {
                Ok(c) => c,
                Err(e) => return Err(e.to_string())
            };
        {
            let mut data = discord_client.data.write().await;

            // this is where we store the role, and the users who have that role
            data.insert::<DesiredMembers>(IndexMap::default());

            // this is where we store the Shard Manager, which allows the bot to shutdown once its done
            data.insert::<ShardManagerContainer>(discord_client.shard_manager.clone());
        }

        // start and wait for the bot to shutdown
        println!("Starting Discord client..");
        match discord_client.start().await {
            Ok(_) => {
                // if shutdown went okay, then attempt to find our data
                let data = discord_client.data.read().await;
                match data.get::<DesiredMembers>(){
                    Some(m) => {
                        if m.is_empty(){
                            return Err("No members were found within any of the desired roles.".to_string())
                        }
                        return Ok(m.clone())
                    },
                    None => return Err("List of found members does not exist as a data entry. This should never happen, please report this!".to_string())
                }
            },
            Err(e) => return Err(e.to_string())
        }
    })
}

/// Removes special characters present in usernames.
fn strip_special_characters(word: String) -> String{
    word.replace("_", "").replace(".", "")
}

/// Returns true if from contains at least one matching element from to.
fn shared_elements<T: Eq>(from: &Vec<T>, to: &Vec<T>) -> bool{
    for n in 0..from.len() {
        if to.contains(from.get(n).unwrap()){
            return true;
        }
    }
    return false;
}

/// Attempts to insert a vector of members for a given role into the DesiredMembers data path. Panics if DesiredMembers doesn't exist in the context data.
async fn insert_members (ctx: &Context, role: String, members: Vec<String>){
    let mut data = ctx.data.write().await;

    let desired_members =  match data.get_mut::<DesiredMembers>(){
        Some(m) => m,
        None => panic!("List of found members does not exist as a data entry. This should never happen, please report this!")
    };

    let entry = desired_members.entry(role.into()).or_insert(vec![]);
    *entry = members;
}