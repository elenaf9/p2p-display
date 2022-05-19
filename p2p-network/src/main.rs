mod network;
use network::Network;

// Main function that gets executed on `cargo run`.
//
// We annotate this with `#[async_std::main]` because the network interaction is 
// asynchronous (i.e. a future, see https://rust-lang.github.io/async-book/).
//
// The `#[async_std::main]` macro internally spawns a new task that runs 
// our (async) main function.
#[async_std::main]
async fn main() ->  Result<(), Box<dyn std::error::Error>> {
    // All logic is implement in our `network` mod. 
    // Refer to its docs for more info on the below method calls.
    let mut network = Network::new().await?;
    network.start_listening()?;
    network.subscribe()?;
    network.run().await
}
