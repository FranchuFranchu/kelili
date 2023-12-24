local cc_hash = param.cc_hash
local send_hash = param.send_hash
return kelili.io_run_fun(function()
  local marker = IO.mark()
  local root_hash = IO.hash()
  local parent = IO.call(cc_hash)({
    type = "create",
    allow_send = function(transaction, parent_hash)
      -- Doesn't matter, we never send from this account :D
      return false, "Non-sending account"
    end
  })
  return parent
end)
