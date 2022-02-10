pub mod discord_webhook;
pub use discord_webhook::DiscordWebhookFilter;

pub mod warning_filter;
pub use warning_filter::WarningFilter;

pub mod blacklist;
pub use blacklist::BlacklistFilter;

pub mod files;
pub use files::FilesSavingFilter;

pub mod dedupe;
pub use dedupe::DedupeFilter;
