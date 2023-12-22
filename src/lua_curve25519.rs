use std::num::TryFromIntError;

use blake2::Blake2s256;
use curve25519_dalek::{
    edwards::{CompressedEdwardsY, EdwardsPoint},
    scalar::Scalar,
};
use mlua::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[derive(Clone, Debug, FromLua)]
pub struct LuaEdwardsPoint(pub EdwardsPoint);

#[derive(Clone, Debug, FromLua)]
pub struct LuaScalar(pub Scalar);

#[derive(Clone, Debug, FromLua)]
pub struct LuaU256(pub [u8; 32]);

#[derive(Clone, Debug, FromLua)]
pub struct LuaRng(ChaCha20Rng);

impl<'a> TryFrom<mlua::Value<'a>> for LuaScalar {
    type Error = LuaError;
    fn try_from(value: mlua::Value<'a>) -> Result<Self, Self::Error> {
        if let Some(n) = value.as_integer() {
            let n: u64 = n
                .try_into()
                .map_err(|x: TryFromIntError| x.into_lua_err())?;
            return Ok(LuaScalar(Scalar::from(n)));
        }
        if let Some(n) = value.as_number() {
            // TODO try to do this?
            let n: u64 = n as u64;
            return Ok(LuaScalar(Scalar::from(n)));
        }
        if let Some(n) = value
            .as_userdata()
            .map(|s| s.borrow::<LuaScalar>().ok())
            .flatten()
        {
            return Ok(n.clone());
        }
        if let Some(n) = value
            .as_userdata()
            .map(|s| s.borrow::<LuaU256>().ok())
            .flatten()
        {
            return Ok(LuaScalar(Scalar::from_bytes_mod_order(n.0.clone())));
        }
        return Err(LuaError::FromLuaConversionError {
            from: value.type_name(),
            to: "LuaScalar",
            message: None,
        });
    }
}

pub fn mul_fn<'lua>(
    lua: &'lua Lua,
    this: mlua::Value<'lua>,
    other: mlua::Value<'lua>,
) -> LuaResult<mlua::Value<'lua>> {
    if let Some(this) = this
        .as_userdata()
        .map(|s| s.take::<LuaEdwardsPoint>().ok())
        .flatten()
    {
        if let Some(_other) = other
            .as_userdata()
            .map(|s| s.borrow::<LuaEdwardsPoint>().ok())
            .flatten()
        {
            Err("Can't multiply two points together".into_lua_err())
        } else {
            Ok(LuaEdwardsPoint(this.0 * LuaScalar::try_from(other.clone())?.0).into_lua(lua)?)
        }
    } else {
        let this = LuaScalar::try_from(this)?;
        if let Some(other) = other
            .as_userdata()
            .map(|s| s.borrow::<LuaEdwardsPoint>().ok())
            .flatten()
        {
            Ok(LuaEdwardsPoint(this.0 * other.0).into_lua(lua)?)
        } else {
            Ok(LuaScalar(this.0 * LuaScalar::try_from(other.clone())?.0).into_lua(lua)?)
        }
    }
}

impl LuaUserData for LuaScalar {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function("__add", |_, (this, other): (mlua::Value, mlua::Value)| {
            Ok(LuaScalar(
                LuaScalar::try_from(this)?.0 + LuaScalar::try_from(other)?.0,
            ))
        });
        methods.add_meta_function("__sub", |_, (this, other): (mlua::Value, mlua::Value)| {
            Ok(LuaScalar(
                LuaScalar::try_from(this)?.0 - LuaScalar::try_from(other)?.0,
            ))
        });
        methods.add_meta_function("__mul", |lua, (this, other): (mlua::Value, mlua::Value)| {
            mul_fn(lua, this, other)
        });
        methods.add_meta_function("__eq", |_, (this, other): (LuaScalar, LuaScalar)| {
            Ok(this.0 == other.0)
        });
        methods.add_meta_method("__tostring", |_lua, this: &LuaScalar, ()| {
            Ok(format!(
                "crypto.Scalar(0x{})",
                hex::encode(this.0.to_bytes())
            ))
        });
        methods.add_method("__serpent", |_lua, this: &LuaScalar, ()| {
            Ok(format!(
                "crypto.Scalar.deserialize({:?})",
                hex::encode(this.0.to_bytes())
            ))
        });
    }
}
impl LuaUserData for LuaEdwardsPoint {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function(
            "__add",
            |_, (this, other): (LuaEdwardsPoint, LuaEdwardsPoint)| {
                Ok(LuaEdwardsPoint(this.0 + other.0))
            },
        );
        methods.add_meta_function(
            "__sub",
            |_, (this, other): (LuaEdwardsPoint, LuaEdwardsPoint)| {
                Ok(LuaEdwardsPoint(this.0 - other.0))
            },
        );
        methods.add_meta_function(
            "__eq",
            |_, (this, other): (LuaEdwardsPoint, LuaEdwardsPoint)| Ok(this.0 == other.0),
        );
        methods.add_meta_function("__mul", |lua, (this, other): (mlua::Value, mlua::Value)| {
            mul_fn(lua, this, other)
        });

        methods.add_meta_method("__tostring", |_lua, this: &LuaEdwardsPoint, ()| {
            Ok(format!(
                "crypto.Point(0x{})",
                hex::encode(this.0.compress().to_bytes())
            ))
        });
        methods.add_method("__serpent", |_lua, this: &LuaEdwardsPoint, ()| {
            Ok(format!(
                "crypto.Point.deserialize({:?})",
                hex::encode(this.0.compress().to_bytes())
            ))
        })
    }
}

impl LuaUserData for LuaU256 {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("__serpent", |_lua, this: &LuaU256, ()| {
            Ok(format!(
                "crypto.U256.deserialize({:?})",
                hex::encode(this.0)
            ))
        });
        methods.add_meta_method("__tostring", |_lua, this: &LuaU256, ()| {
            Ok(format!("crypto.U256(0x{})", hex::encode(this.0)))
        });
        methods.add_meta_function("__eq", |_, (this, other): (LuaU256, LuaU256)| {
            Ok(this.0 == other.0)
        });
    }
}

impl LuaUserData for LuaRng {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {}
}

use group::Group;
use group::{ff::PrimeField, GroupEncoding};

pub fn make_lib<'l>(lua: &'l Lua, modname: String) -> LuaResult<mlua::Value<'l>> {
    assert!(modname == "crypto");
    let crypto = lua.create_table()?;
    let scalar = lua.create_table()?;
    let point = lua.create_table()?;
    let random = lua.create_table()?;
    let u256 = lua.create_table()?;
    crypto.set("Scalar", scalar.clone())?;
    crypto.set("Point", point.clone())?;
    crypto.set("Random", random.clone())?;
    crypto.set("U256", u256.clone())?;
    scalar.set(
        "random",
        LuaFunction::wrap(|_lua, mut r: LuaRng| Ok(LuaScalar(Scalar::random(&mut r.0)))),
    )?;
    scalar.set(
        "identity",
        LuaFunction::wrap(|_lua, ()| Ok(LuaScalar(Scalar::ONE))),
    )?;
    scalar.set(
        "zero",
        LuaFunction::wrap(|_lua, ()| Ok(LuaScalar(Scalar::ZERO))),
    )?;
    scalar.set(
        "generator",
        LuaFunction::wrap(|_lua, ()| Ok(LuaScalar(Scalar::MULTIPLICATIVE_GENERATOR))),
    )?;
    scalar.set(
        "from",
        LuaFunction::wrap(|_lua, v: mlua::Value| LuaScalar::try_from(v)),
    )?;
    scalar.set(
        "deserialize",
        LuaFunction::wrap(|_lua, code: String| {
            Ok(LuaScalar(Scalar::from_bytes_mod_order(
                hex::decode(&code).unwrap().try_into().unwrap(),
            )))
        }),
    )?;
    point.set(
        "random",
        LuaFunction::wrap(|_lua, mut r: LuaRng| {
            Ok(LuaEdwardsPoint(EdwardsPoint::random(&mut r.0)))
        }),
    )?;
    point.set(
        "identity",
        LuaFunction::wrap(|_lua, ()| Ok(LuaEdwardsPoint(<EdwardsPoint as Group>::identity()))),
    )?;
    point.set(
        "generator",
        LuaFunction::wrap(|_lua, ()| Ok(LuaEdwardsPoint(<EdwardsPoint as Group>::generator()))),
    )?;
    point.set(
        "deserialize",
        LuaFunction::wrap(|_lua, code: String| {
            Ok(LuaEdwardsPoint(
                (CompressedEdwardsY::from_slice(&hex::decode(&code).unwrap()).unwrap())
                    .decompress()
                    .unwrap(),
            ))
        }),
    )?;
    random.set(
        "from_entropy",
        LuaFunction::wrap(|_lua, ()| Ok(LuaRng(ChaCha20Rng::from_entropy()))),
    )?;
    u256.set(
        "hash",
        LuaFunction::wrap(|_lua, code: String| {
            use blake2::Digest;
            let mut rng = <Blake2s256 as Digest>::new();
            rng.update(code);
            let mut s = LuaU256([0; 32]);
            Digest::finalize_into(rng, (&mut s.0).into());
            Ok(s)
        }),
    )?;
    u256.set(
        "deserialize",
        LuaFunction::wrap(|_lua, code: String| {
            Ok(LuaU256(hex::decode(&code).unwrap().try_into().unwrap()))
        }),
    )?;
    Ok(crypto.into_lua(lua)?)
}
