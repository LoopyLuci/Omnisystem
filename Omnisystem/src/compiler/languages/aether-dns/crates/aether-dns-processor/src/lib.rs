/// DNS Query Processor
/// Multi-stage pipeline for processing DNS queries

pub mod processor;
pub mod pipeline;
pub mod upstream;
pub mod response_builder;

pub use processor::QueryProcessor;
pub use pipeline::{ProcessingStage, ProcessingContext};
pub use upstream::UpstreamResolver;
pub use response_builder::ResponseBuilder;
