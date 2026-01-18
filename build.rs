fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("task_descriptor.bin"))
        .compile_protos(&["proto/task/task.proto"], &["proto"])?;

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("user_descriptor.bin"))
        .compile_protos(&["proto/user/user.proto"], &["proto"])?;

    Ok(())
}
