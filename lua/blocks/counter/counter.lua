return kelili.io_run_fun(function()
  local marker = IO.mark()
  function mk_update(state)
    return function(msg)
      return {
        update = mk_update(state + 1)
        count = marker(state.count + 1)
      }
    end
  end
  return {update = mk_update(0), count = 0}
end)
