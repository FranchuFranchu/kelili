local parent_hash = param.parent_hash
local signature = param.signature
local dest = param.dest
local amount = param.amount
return kelili.io_run_fun(function()
  local state = IO.call_bubble(parent_hash)
  local w = IO.bubble(state.state({
    type = "send",
    dest = param.dest,
    amount = param.amount,
    signature = param.signature
  }))
  return w
end)
