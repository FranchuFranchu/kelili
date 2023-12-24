use std::error::Error;

use mlua::prelude::*;

use crate::{dht::Peer, lua_curve25519::LuaU256};

use super::types::Id;

pub type Code = Box<[u8]>;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub index: u64,
    pub mana_limit: u64,
    pub memo_limit: u64,
    pub code: Code,
    pub name: Option<String>,
}

pub struct Node {
    pub request_dht: crate::dht::Peer,
    pub node_dht: crate::dht::Peer,
}

pub struct Context {
    pub block_hash: Id,
    pub remaining_mana: u64,
    pub remaining_memo: u64,
}

use async_recursion::async_recursion;
impl Node {
    pub fn new(rng: &mut dyn rand::RngCore) -> Self {
        Self {
            request_dht: Peer::new(rng),
            node_dht: Peer::new(rng),
        }
    }
    pub async fn get_block(&mut self, hash: &Id) -> Result<Option<Block>, Box<dyn Error>> {
        Ok(self
            .request_dht
            .find(hash)
            .await?
            .map(|data: Box<[u8]>| -> Result<Block, Box<dyn Error>> {
                let data = bincode::deserialize(&*data)?;
                Ok(data)
            })
            .transpose()?)
    }

    #[async_recursion(?Send)]
    pub async fn exec_io<'lua>(
        &mut self,
        lua: &'lua Lua,
        context: &mut Context,
        io: mlua::Value<'lua>,
    ) -> Result<mlua::Value<'lua>, Box<dyn Error>> {
        #[derive(Clone, FromLua)]
        pub struct MarkedTerm {
            pub hash: Id,
        }

        let io = match io {
            mlua::Value::Table(x) => x,
            _ => return Err("Expected table!".into()),
        };
        match io.get::<&str, String>("type")?.as_ref() {
            "call" => {
                let cont = io.get::<&str, mlua::Function>("cont")?;
                let hasht = io.get::<&str, LuaU256>("hash")?;
                let _max_mana = io.get::<&str, u64>("max_mana")?;
                let _max_memo = io.get::<&str, u64>("max_memo")?;
                let hash = hasht.0;
                let ret = self.run_block(lua, &hash).await?;
                self.exec_io(lua, context, cont.call(ret)?).await
            }
            "mark" => {
                let cont = io.get::<&str, mlua::Function>("cont")?;
                let hash = context.block_hash.clone();
                let f = LuaFunction::wrap(move |lua, uv: LuaValue| {
                    let udata = lua.create_any_userdata(MarkedTerm { hash })?;
                    udata.set_user_value(uv)?;
                    Ok(udata)
                });
                self.exec_io(lua, context, cont.call(f)?).await
            }
            "open" => {
                let marked: LuaValue = io.get("marked")?;
                let cont = io.get::<&str, mlua::Function>("cont")?;
                let hash;
                let uv: LuaValue;
                if let Some(marked) = match marked.clone() {
                    mlua::Value::UserData(x) => Some(x),
                    _ => None,
                } {
                    uv = marked.user_value()?;
                    let marked: MarkedTerm = marked.take()?;
                    hash = LuaU256(marked.hash).into_lua(lua)?;
                } else {
                    uv = marked.clone();
                    hash = LuaValue::Nil;
                };
                self.exec_io(lua, context, cont.call((uv, hash))?).await
            }
            "done" => Ok(io.get::<&str, mlua::Value>("value")?),
            x => return Err(format!("Invalid type {:?}", x).into()),
        }
    }
    pub async fn run_block<'lua>(
        &mut self,
        lua: &'lua Lua,
        hash: &Id,
    ) -> Result<mlua::Value<'lua>, Box<dyn Error>> {
        let cache: mlua::Table = lua.named_registry_value("kelili.state_cache")?;
        let cache_key = hash.to_string().as_str().into_lua(lua)?;
        let cached = cache.get(cache_key.clone())?;
        if let LuaValue::Table(cached) = cached {
            println!(
                "Fetching {:?} from cache ",
                cached.get::<&str, mlua::Value>("name")?.to_string()?
            );
            Ok(cached.get("value")?)
        } else {
            let block = self.get_block(hash).await?.unwrap();
            let mut ctx = Context {
                block_hash: hash.clone(),
                remaining_mana: block.mana_limit,
                remaining_memo: block.memo_limit,
            };
            let f: LuaFunction = lua.named_registry_value("kelili.stdlib").unwrap();
            let _: () = f.call(())?;
            let mut code = lua.load(&*block.code);
            if let Some(ref name) = block.name {
                code = code.set_name(name);
            }
            println!("Running {:?}", &block.name);
            let val = self.exec_io(lua, &mut ctx, code.call(()).unwrap()).await?;
            cache.set(
                cache_key.clone(),
                lua.create_table_from([
                    ("name", block.name.into_lua(lua)?),
                    ("value", val.clone().into_lua(lua)?),
                ])?,
            )?;
            Ok(val)
        }
    }
}
