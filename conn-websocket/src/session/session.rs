use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Copy, Hash, Eq)]
pub enum UserType {
    CustomerService,
    Customer,
}

#[derive(Clone, Debug, PartialEq, Eq)]

/// Identity is a conn identity.
pub struct Identity(UserType, String);

impl Hash for Identity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{}-{}", self.0 as u8, self.1).hash(state);
    }
}

impl Identity {
    pub fn new(user_type: UserType, identity: String) -> Self {
        Identity(user_type, identity)
    }

    pub fn user_type(&self) -> UserType {
        self.0
    }

    pub fn identity(&self) -> &str {
        &self.1
    }

    pub fn is_customer_service(&self) -> bool {
        self.0 == UserType::CustomerService
    }

    pub fn is_customer(&self) -> bool {
        self.0 == UserType::Customer
    }
}

/// RoomId is a chat room identity.
pub type RoomId = String;
