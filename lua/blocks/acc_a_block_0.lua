local parent_hash = param.cc_hash
local pk = param.pk
return kelili.io_run_fun(function()
  return IO.call(parent_hash)(pk)
end)
