pub fn id() -> std::io::Result<String> {
    std::fs::read_to_string("/etc/machine-id").map(|id| id.trim().to_owned())
}
