use anyhow::Result;
pub struct Bluetooth{
    adapter: bluer::Adapter,
    session: bluer::Session,
}

impl Bluetooth {
    pub fn new() -> Result<Self> {
        let session = bluer::Session::new().await?;
        Self
    }
    pub fn listen(self) -> Result<()> {
        Ok(())
    }
}
