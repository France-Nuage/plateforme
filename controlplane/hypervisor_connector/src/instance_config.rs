pub struct InstanceConfig {
    /// The instance unique id.
    pub id: String,

    /// The number of cores per socket.
    pub cores: u8,

    /// The disk image to create the instance from.
    pub disk_image: String,

    /// Memory properties.
    pub memory: u32,

    /// The instance human-readable name.
    pub name: String,

    /// The Cloud-Init snippet.
    pub snippet: String,
}
