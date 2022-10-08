use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure()
        .type_attribute(".", "#[allow(clippy::all)]")
        .compile(&["proto/api.proto"], &["proto"])?;

    Ok(())
}

