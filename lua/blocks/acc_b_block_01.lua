local cc_hash = param.cc_hash
local send_hash = param.send_hash
return kelili.io_run_fun(function()
  local pk = 42
  local state = IO.call(cc_hash)(pk)
  local recv = state.state({
    type = "receive",
    from = send_hash,
  })
  return recv
end)