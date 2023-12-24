local parent_hash = param.parent_hash
local signature = param.signature
local dest = param.dest
local amount = param.amount
return kelili.io_run_fun(function()
  local parent = IO.call_bubble(parent_hash)
  local w = IO.bubble(parent.update({
    type = "send",
    amount = param.amount,
    dest = param.dest,
    parent_hash = parent_hash,
    signature = signature,
  }))
  return w
end)

