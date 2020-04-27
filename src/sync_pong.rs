use std::error::Error;
mod sync_lib;
use sync_lib::{Ctx, CtxOptions};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating a synchronous mqtt client...");
    
    let mut ctx_options = CtxOptions::new_defaults();
    ctx_options.set_host("test.mosquitto.org:1883");
    ctx_options.set_client_id("Synchronised ponger");
    ctx_options.set_subscribed_topic("ping-ask");
    ctx_options.set_publishing_topic("pong-response");
    ctx_options.set_last_will_and_testament("the synchronised ponger lost the connection");
    ctx_options.set_clean_session(true);
    ctx_options.set_quality_of_service(2);

    let mut ctx = Ctx::create_context(ctx_options)?;

    ctx.establish_connection()?;

    loop {
        if ctx.received("ping")? {
            ctx.publish("pong")?;
            println!("Our job here is done");
            break;
        }
        ctx.reestablish_connection();
    }

    ctx.client.disconnect(None)?;

    Ok(())
}
