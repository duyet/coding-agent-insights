//! Web server module

use cai_core::Result;
use cai_storage::Storage;

/// Run the web server
pub async fn run<S>(_storage: std::sync::Arc<S>, _config: super::Config) -> Result<()>
where
    S: Storage + ?Sized,
{
    Ok(())
}
