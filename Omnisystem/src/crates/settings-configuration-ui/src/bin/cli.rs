//! Demo CLI: resolve a "theme" setting across default/project/user layers.

use settings_configuration_ui::{resolve_all, Layer, LayerValues, Schema, SettingType, SettingValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut schema = Schema::new();
    schema.insert("theme".to_string(), SettingType::String);

    let mut defaults = LayerValues::new();
    defaults.insert("theme".to_string(), SettingValue::Str("light".into()));
    let mut user = LayerValues::new();
    user.insert("theme".to_string(), SettingValue::Str("dark".into()));

    let layers: Vec<(Layer, &LayerValues)> = vec![(Layer::Default, &defaults), (Layer::User, &user)];
    let resolved = resolve_all(&schema, &layers)?;
    println!("resolved theme: {:?}", resolved.get("theme"));
    Ok(())
}
