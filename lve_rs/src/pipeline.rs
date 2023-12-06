use anyhow::Result;
use std::fs::File;

pub struct Pipeline {}

impl Pipeline {
    pub fn new(vert_file_path: &str, frag_file_path: &str) -> Result<Self> {
        Self::create_graphics_pipeline(vert_file_path, frag_file_path)?;

        Ok(Self {})
    }

    fn read_file(file_path: &str) -> Result<File> {
        Ok(File::open(file_path)?)
    }

    /* --- Helper functions --- */
    fn create_graphics_pipeline(vert_file_path: &str, frag_file_path: &str) -> Result<()> {
        let vert_code = Self::read_file(vert_file_path)?;
        let frag_code = Self::read_file(frag_file_path)?;

        println!("Vertex shader code size: {}", vert_code.metadata()?.len());
        println!("Fragment shader code size: {}", frag_code.metadata()?.len());

        Ok(())
    }
}
