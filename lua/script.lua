local serpent = require("lua/serpent")
local crypto = require("crypto")
local crypto_util = require("lua/crypto_util")

local p = crypto.Point.identity()
local a = serpent.dump(p)

function block_from_file(name, param)
  local f = io.open("lua/blocks/" .. name)
  local code = f:read("*a")
  f:close()
  local s = serpent.dump(param)
  local s = string.gsub(s, "\\", "\\\\")
  local s = string.gsub(s, "\"", "\\\"")
  local code = "local param = loadstring(\"" .. s .. "\")()\n" .. code
  local code = string.dump(load(code))
  return node:new_block(code, name)
end

local sk = crypto_util.gen_sk()
local signature = sk:sign("Hi")

local catcoin_hash = block_from_file("catcoin.lua")
node:run_block(catcoin_hash)
local acc_a_block_0_hash = block_from_file("acc_a_block_0.lua", {pk = sk:pk(), cc_hash = catcoin_hash})
local result = node:run_block(acc_a_block_0_hash)
if result.error then
  kelili.debug(result)
  return
end


local acc_a_block_1_hash = block_from_file("acc_a_block_1.lua", 
  {
    parent_hash = acc_a_block_0_hash, 
    dest = 42,
    amount = 16,
    signature = sk:sign({parent_hash = acc_a_block_0_hash, dest = 42, amount = 16})
  })
local result = node:run_block(acc_a_block_1_hash)
if result.error then
  kelili.debug(result)
  return
end

local acc_b_block_01_hash = block_from_file("acc_b_block_01.lua", {cc_hash = catcoin_hash, send_hash = acc_a_block_1_hash})
local result = node:run_block(acc_b_block_01_hash)
if result.error then
  kelili.debug(result)
  return
end