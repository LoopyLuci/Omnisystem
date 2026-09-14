/// Compute Workers - 18+ CPU-Intensive Operations

pub mod cpu_intensive;
pub mod sorting;
pub mod hashing;
pub mod encryption;
pub mod json_parse;
pub mod regex;
pub mod parallel_map;
pub mod decompression;
pub mod xml_parse;
pub mod yaml_parse;
pub mod compression;
pub mod matrix;
pub mod image_processing;
pub mod anomaly;
pub mod streaming;
pub mod serialization;
pub mod sorting_advanced;

pub use cpu_intensive::CPUIntensiveWorker;
pub use sorting::SortWorker;
pub use hashing::ComputeHashingWorker;
pub use encryption::EncryptionWorker;
pub use json_parse::JSONParseWorker;
pub use regex::RegexWorker;
pub use parallel_map::ParallelMapWorker;
pub use decompression::DecompressionWorker;
pub use xml_parse::XMLParseWorker;
pub use yaml_parse::YAMLParseWorker;
pub use compression::CompressionWorker;
pub use matrix::MatrixWorker;
pub use image_processing::ImageProcessingWorker;
pub use anomaly::AnomalyWorker;
pub use streaming::StreamingWorker;
pub use serialization::SerializationWorker;
pub use sorting_advanced::AdvancedSortWorker;
