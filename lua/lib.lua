crypto = require("crypto")
serpent = require("lua/serpent")
local function run_fun(f)
  return kelili.io_run_coro(coroutine.create(function(...) return "done", {value = f(...)} end))
end

local function debug(...)
  print(require("lua/serpent").block(...))
end

IO = {
  call = function(hash, max_mana, max_memo)
    return coroutine.yield("call", {
      hash = hash,
      max_mana = max_mana,
      max_memo = max_memo
    })
  end,
  done = function(value)
    return coroutine.yield("done", {value = value})
  end,
  bubble = function(v)
    if v.error then
      IO.done({error = {message = "Had an error", data = v}})
    end
    return v
  end,
  call_bubble = function(hash, max_mana, max_memo)
    local ret = IO.call(hash, max_mana, max_memo)
    if ret.error then
      IO.done({error = {message = "Had an error while calling " .. hash:serialize(), data = ret}})
    end
    return ret
  end,
  mark = function()
    return coroutine.yield("mark")
  end,
  open = function(e)
    return coroutine.yield("open", {marked = e})
  end,
  hash = function()
    local marker = IO.mark()
    local obj, hash = IO.open(marker(nil))
    return hash
  end
}

local function run_coro(co, ...)
  local succ, t, action = coroutine.resume(co, ...)
  if action == nil then
    action = {}
  end
  -- print(t, action, action.value)
  return {
    type = t,
    value = action.value,
    marked = action.marked,
    hash = action.hash,
    max_mana = action.max_mana or 0,
    max_memo = action.max_memo or 0,
    cont = function(...)
      return run_coro(co, ...)
    end
  }
end

--- deeply compare two objects
local function equals(o1, o2, ignore_mt)
  -- same object
  if o1 == o2 then return true end

  local o1Type = type(o1)
  local o2Type = type(o2)
  --- different type
  if o1Type ~= o2Type then return false end
  --- same type but not table, already compared above
  if o1Type ~= 'table' then return false end

  -- use metatable method
  if not ignore_mt then
    local mt1 = getmetatable(o1)
    if mt1 and mt1.__eq then
      --compare using built in method
      return o1 == o2
    end
  end

  -- iterate over o1
  for key1, value1 in pairs(o1) do
    local value2 = o2[key1]
    if value2 == nil or equals(value1, value2, ignore_mt) == false then
      return false
    end
  end

  --- check keys in o2 but missing from o1
  for key2, _ in pairs(o2) do
    if o1[key2] == nil then return false end
  end
  return true
end

kelili = {
  equals = equals,
  io_run_coro = run_coro,
  io_run_fun = run_fun,
  debug = debug,
}
