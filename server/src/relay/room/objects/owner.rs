use crate::relay::room::clients::Client;
use crate::relay::room::room::ClientId;

/// владелец
/// или клиент или root
pub struct Owner {
	client: Option<ClientId>
}


const ROOT_OWNER: Owner = Owner { client: Option::None };

impl Owner {
	pub fn new_root_owner() -> Owner {
		ROOT_OWNER
	}
	
	pub fn new_owner(client: &Client) -> Owner {
		Owner { client: Option::Some(client.configuration.id) }
	}
}

impl PartialEq for Owner {
	fn eq(&self, other: &Self) -> bool {
		return self.client.unwrap_or_default() == other.client.unwrap_or_default();
	}
}

impl Eq for Owner {}

