use crate::{
    cell_type::{CellType, CellTypeMap},
    matrix::vec_matrix::VecMatrix,
    rules::RuleSet,
    ConvolutionMatrix, FieldType,
};
use flate2::{write::ZlibEncoder, Compression, read::ZlibDecoder};
use std::io::{Write, Read};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum ConvMatrixE<const CW: usize> {
    Single(ConvolutionMatrix<CW>),
    Multiple([ConvolutionMatrix<CW>; 9]),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SaveFile<const CW: usize> {
    pub convolution: Option<ConvMatrixE<CW>>,
    pub rules: Option<RuleSet<FieldType>>,
    pub cell_type_map: Option<CellTypeMap>,
    pub cells: Option<Vec<VecMatrix<CellType>>>,
    pub include_convolution: bool,
    pub include_rules: bool,
    pub include_cell_type_map: bool,
    pub include_cells: bool,
}

impl<const CW: usize> SaveFile<CW> {
    pub fn save_to(&self, filename: &str) -> Result<(), bincode::Error> {
        let bytes = bincode::serialize(self)?;
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(&bytes)?;
        let mut file = std::fs::File::create(filename)?;
        let encoded = e.finish()?;
        file.write_all(&encoded).map_err(|err| Box::new(bincode::ErrorKind::Io(err)))
    }

    pub fn load_from_bytes(encoded: &[u8]) -> Result<SaveFile<CW>, bincode::Error> {
        let mut decoder = ZlibDecoder::new(encoded);
        let mut serialized_bytes = Vec::new();
        decoder.read_to_end(&mut serialized_bytes)?;
        bincode::deserialize::<Self>(&serialized_bytes)
    }
}
