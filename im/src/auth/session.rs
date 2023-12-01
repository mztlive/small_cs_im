use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Copy, Hash, Eq, Deserialize, Serialize)]
pub enum UserType {
    CustomerService,
    Customer,
}

/// Member is a struct wrapper for connection identity.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Member {
    user_name: String,
    user_type: UserType,
    id: String,
}

impl Hash for Member {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{}-{:?}", self.id, self.user_type).hash(state);
    }
}

impl Member {
    pub fn new(user_type: UserType, id: String, user_name: String) -> Self {
        Member {
            user_name,
            user_type,
            id,
        }
    }

    pub fn user_type(&self) -> UserType {
        self.user_type
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_customer_service(&self) -> bool {
        self.user_type == UserType::CustomerService
    }

    pub fn is_customer(&self) -> bool {
        self.user_type == UserType::Customer
    }

    pub fn user_name(&self) -> &str {
        &self.user_name
    }
}

/// RoomId is a chat room identity.
pub type RoomId = String;
