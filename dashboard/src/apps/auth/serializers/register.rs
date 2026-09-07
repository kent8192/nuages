//! Register request serializer.

use reinhardt::dto;
use reinhardt::pages::client_form;
use serde::{Deserialize, Serialize};

/// User registration request body.
#[dto(schema)]
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[client_form(server_fn = crate::apps::auth::server_fn::register::register, validate)]
pub struct RegisterRequest {
	#[validate(length(min = 3, max = 32))]
	pub username: String,
	#[validate(email, length(max = 254))]
	pub email: String,
	#[validate(length(min = 8, max = 128))]
	pub password: String,
}
