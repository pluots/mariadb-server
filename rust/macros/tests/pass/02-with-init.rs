include!("../include.rs");

register_plugin! {
    TestPlugin,
    ptype: PluginType::MariaEncryption,
    name: "debug_key_management",
    author: "Trevor Gross",
    description: "Debug key management plugin",
    license: License::Gpl,
    maturity: Maturity::Experimental,
    version: "0.1",
    init: TestPlugin,
    encryption: false,
}

fn main() {
    use mariadb::bindings::st_maria_plugin;

    let plugin_def: &st_maria_plugin = unsafe { &*(_maria_plugin_declarations_[0]).get() };

    assert!(plugin_def.init.is_some());
    assert!(plugin_def.deinit.is_some());
}
