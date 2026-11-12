mod external_url;
mod link_action;
mod mud_link;

pub use external_url::{is_external_url, origin_of, scheme_of, target_opens_new_tab};
pub use link_action::LinkAction;
pub use mud_link::{LinkResponse, MudLink};