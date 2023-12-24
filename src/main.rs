#![feature(btree_cursors)]

use std::sync::{Arc, Mutex};

use mlua::{Function, IntoLua, Lua, Value};

use rand::SeedableRng;

use crate::{lua_curve25519::make_lib, node::Node, script_vm::NodeLock};
pub mod dht;
pub mod lua_curve25519;
pub mod node;
pub mod script_vm;
pub mod types;

#[derive(clap::Parser, Debug)]
#[command(
    author = "FranchuFranchu", 
    version,
    about = "A currencyless decentralized computer",
    long_about = None)]
pub struct Cli {
    script: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::*;
    let cli = Cli::parse();

    let lua = Lua::new();

    let std = std::fs::read("lua/lib.lua")?;
    let std: Function = lua.load(std).into_function()?;
    lua.set_named_registry_value("kelili.stdlib", std)?;
    lua.set_named_registry_value("kelili.state_cache", lua.create_table()?)?;
    let f = mlua::prelude::LuaFunction::wrap(|l, m| make_lib(l, m)).into_lua(&lua)?;
    let _v = lua.load_from_function::<Value>("crypto", f.as_function().unwrap().clone());

    let mut rng = rand_chacha::ChaCha12Rng::from_entropy();

    let node = Node::new(&mut rng);
    let node = NodeLock(Arc::new(Mutex::new(node)));
    lua.globals().set("node", node)?;

    let script = cli
        .script
        .as_ref()
        .map(|x| x.as_ref())
        .unwrap_or("lua/script.lua");
    let code = std::fs::read(script)?;
    lua.load(code).set_name(script).exec()?;

    Ok(())
}
