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
    variables: [
        SysVar {
            ident: _SYSVAR_CONST_STR,
            vtype: SysVarConstString,
            name: "test_sysvar",
            description: "this is a description",
            options: [SysVarOpt::ReadOnly, SysVarOpt::NoCliOption],
            default: "default value",
            interval: "50"
        }
    ]
}

fn main() {}
