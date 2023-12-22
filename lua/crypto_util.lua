local crypto = require("crypto")
local serpent = require("lua/serpent")

local rng = crypto.Random.from_entropy()

function lua_to_scalar(obj)
  return crypto.Scalar.from(crypto.U256.hash(require("lua/hash").hash(obj)))
end

pk_metatable = {
  __index = {
    as_scalar = function(self) return self.value end,
    verify = function(self, signature, message)
      local new_challenge = lua_to_scalar({message, signature.response * crypto.Point.generator() + signature.challenge * self:as_scalar()})
      return new_challenge == signature.challenge
    end,
  },
  __serpent = function(self, serialize)
    return "require(\"lua/crypto_util\").set_pk_metatable({ value = " .. serialize(self.value) .. "})"
  end
}

sk_metatable = {
  __index = {
    as_scalar = function(self) return self.value end,
    sign = function(self, message) 
      local alpha = crypto.Scalar.random(rng)
      print(alpha)
      local _ = alpha * crypto.Point.generator()
      print(alpha)
      local challenge = lua_to_scalar({message, alpha * crypto.Point.generator()})
      local response = alpha - challenge * self:as_scalar()
      return {
        challenge = challenge,
        response = response,
      }
    end,
    pk = function(self)
      return set_pk_metatable({ value = self:as_scalar() * crypto.Point.generator() })
    end
  }
}


function gen_sk()
  return set_sk_metatable({ value = crypto.Scalar.random(rng) })
end

function set_sk_metatable(t)
  setmetatable(t, sk_metatable)
  return t
end

function set_pk_metatable(t)
  setmetatable(t, pk_metatable)
  return t
end
return {
  gen_sk = gen_sk,
  set_pk_metatable = set_pk_metatable,
  set_sk_metatable = set_sk_metatable,
}