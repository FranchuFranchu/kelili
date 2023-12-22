use crate::{
    lua_curve25519::LuaU256,
    node::{Block, Node},
};
use mlua::prelude::*;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

#[derive(FromLua, Clone)]
pub struct NodeLock(pub Arc<Mutex<Node>>);

impl mlua::UserData for NodeLock {
    fn add_methods<'outer, M: LuaUserDataMethods<'outer, Self>>(methods: &mut M) {
        methods.add_method(
            "new_block",
            |_lua, node, (code, name): (bstr::BString, Option<String>)| {
                let h = Runtime::new().unwrap().block_on(async {
                    let mut node = node.0.lock().unwrap();
                    let block = Block {
                        index: 0,
                        mana_limit: 0,
                        memo_limit: 0,
                        code: code.to_vec().into_boxed_slice(),
                        name,
                    };
                    let q = bincode::serialize(&block).unwrap().into_boxed_slice();
                    let h = node.request_dht.hash(&q);
                    node.request_dht.store(q).await.unwrap();
                    return LuaU256(h);
                });
                Ok(h)
            },
        );
        methods.add_method(
            "run_block",
            |lua, node, (hasht, _param): (LuaU256, mlua::Value)| {
                let ret = Runtime::new().unwrap().block_on(async {
                    let mut node = node.0.lock().unwrap();
                    node.run_block(lua, &hasht.0).await.unwrap()
                });
                Ok(ret)
            },
        );
    }
}
