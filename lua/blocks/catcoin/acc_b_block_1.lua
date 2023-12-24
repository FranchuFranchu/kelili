local parent_hash = param.parent_hash
local send_hash = param.send_hash
return kelili.io_run_fun(function()
  local parent = IO.call(parent_hash)
  local recv = parent.update({
    type = "receive",
    from = send_hash,
  })
  return recv
end)