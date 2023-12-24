-- A smart contract that delays a CatCoin 
-- transaction until the Counter 
-- SC is greater than some value.
return kelili.io_run_fun(function()
  local marker = IO.mark()
  local root_hash = IO.hash()
  local wallet = IO.call_bubble(cc_hash)({
    type = "create",
    allow_send = function(transaction, parent_hash)
      local transaction, hash = IO.open(transaction)
      if hash ~= root_hash then
        return false, "Incorrect hash!"
      end
      if transaction.parent_hash ~= parent_hash then
        return false, "Transaction goes after wrong transaction!"
      end
      return true, transaction
    end
  })
  function mk_update(state)
    return function(msg)
      if msg.type == "register" then
        local new_state = state.wallet.update({
          type = "receive",
          from = msg.from,
        })
        if new_state.error then
          return new_state
        end
        state.wallet = new_state
        state.waiting[#state.waiting+1] = {
          allow = msg.allow,
          after = msg.after,
          amount = new_state.transaction.amount
        }

      end
      if msg.type == "claim" then
        local counter = IO.call_bubble(msg.counter)
      end
    end
  end
  return {update = mk_update({waiting={}, wallet = wallet})}
end)
