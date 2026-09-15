//! Deployment wizard types: an ordered sequence of steps, each requiring
//! certain answer fields before the wizard can advance past it.

use serde::{Deserialize, Serialize};

/// One step in the wizard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardStep {
    /// Step name, e.g. `target-environment`.
    pub name: String,
    /// Answer keys that must be present (and non-empty) to leave this step.
    pub required_fields: Vec<String>,
}

/// A wizard definition: its steps in display order.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WizardDefinition {
    /// The steps, in the order they're presented.
    pub steps: Vec<WizardStep>,
}
