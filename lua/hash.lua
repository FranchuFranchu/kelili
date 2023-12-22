--[[
Ordered table iterator, allow to iterate on the natural order of the keys of a
table.
 
Example:
]]

function __genOrderedIndex(t)
  local orderedIndex = {}
  for key in pairs(t) do
    table.insert(orderedIndex, key)
  end
  table.sort(orderedIndex)
  return orderedIndex
end

function orderedNext(t, state)
  -- Equivalent of the next function, but returns the keys in the alphabetic
  -- order. We use a temporary ordered key table that is stored in the
  -- table being iterated.

  local key = nil
  --print("orderedNext: state = "..tostring(state) )
  if state == nil then
    -- the first time, generate the index
    t.__orderedIndex = __genOrderedIndex(t)
    key = t.__orderedIndex[1]
  else
    -- fetch the next value
    for i = 1, table.getn(t.__orderedIndex) do
      if t.__orderedIndex[i] == state then
        key = t.__orderedIndex[i + 1]
      end
    end
  end

  if key then
    return key, t[key]
  end

  -- no more value to return, cleanup
  t.__orderedIndex = nil
  return
end

function orderedPairs(t)
  -- Equivalent of the pairs() function on tables. Allows to iterate
  -- in order
  return orderedNext, t, nil
end

-- Return a string encoding object data.
function hash(val)
  if type(val) == "userdata" then
    local so, sr = pcall(function() return val:__serpent(hash) end)
    if so then
      return "userdata(" .. sr .. ")"
    else
      return "userdata(" .. tostring(val) .. ")"
    end
  elseif type(val) == "table" then
    s = "table("
    for k, v in orderedPairs(val) do
      s = s .. hash(k) .. "=" .. hash(v)
    end
    return s .. ")"
  else
    return type(val) .. "(" .. tostring(val) .. ")"
  end
end

return {
  hash = hash
}
