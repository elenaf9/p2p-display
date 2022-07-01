mod management;
mod upgrade;

pub type Management = management::Management<p2p_network::NetworkComponent>;
pub use management::UserCommand;
