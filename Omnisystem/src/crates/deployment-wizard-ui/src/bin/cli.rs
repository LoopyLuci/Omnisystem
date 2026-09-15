//! Demo CLI: walk a 2-step deployment wizard to completion.

use deployment_wizard_ui::{WizardDefinition, WizardState, WizardStep};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let def = WizardDefinition {
        steps: vec![
            WizardStep { name: "target".into(), required_fields: vec!["environment".into()] },
            WizardStep { name: "review".into(), required_fields: vec![] },
        ],
    };
    let mut wizard = WizardState::new(def)?;
    println!("step: {} (progress {:.0}%)", wizard.current_step().name, wizard.progress() * 100.0);
    wizard.set_answer("environment", "production");
    wizard.advance()?;
    println!("step: {} (progress {:.0}%)", wizard.current_step().name, wizard.progress() * 100.0);
    Ok(())
}
