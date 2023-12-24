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
  local ok, err = load(code, name)
  if not ok then
    error(err)
  end
  local code = string.dump(ok)
  return node:new_block(code, name)
end


local sk = crypto_util.gen_sk()
local signature = sk:sign("Hi")

local catcoin_hash = block_from_file("catcoin/catcoin.lua")
local acc_a_block_0_hash = block_from_file("catcoin/acc_a_block_0.lua", {cc_hash = catcoin_hash, pk = sk:pk()})

node:run_block(catcoin_hash)
local result = node:run_block(acc_a_block_0_hash)
if result.error then
  kelili.debug(result)
  return
end


local acc_b_block_0_hash = block_from_file("catcoin/acc_b_block_0.lua", {cc_hash = catcoin_hash})

local result = node:run_block(acc_a_block_0_hash)
if result.error then
  kelili.debug(result)
  return
end

local signature = sk:sign({parent_hash = acc_a_block_0_hash, amount = 16, dest = acc_b_block_0_hash})
local acc_a_block_1_hash = block_from_file("catcoin/acc_a_block_1.lua", {
  parent_hash = acc_a_block_0_hash,
  dest = acc_b_block_0_hash,
  amount = 16,
  signature = signature
})

local result = node:run_block(acc_a_block_1_hash)
if result.error then
  kelili.debug(result)
  return
end

local acc_b_block_1_hash = block_from_file("catcoin/acc_b_block_1.lua", {parent_hash = acc_b_block_0_hash, send_hash = acc_a_block_1_hash})

local result = node:run_block(acc_b_block_1_hash)
if result.error then
  kelili.debug(result)
  return
end