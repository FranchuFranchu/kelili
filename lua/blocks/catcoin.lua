return kelili.io_run_fun(function()
  local root_hash = IO.hash()
  local marker = IO.mark()
  function update(state, parent_hash)
    return function(msg)
      print("CatCoin Msg ", require("lua/serpent").line(msg))
      local this_hash = IO.hash()
      if msg.type == "receive" then
        local remote = IO.call_bubble(msg.from)
        local r_transaction, r_hash = IO.open(remote.transaction)
        if not kelili.equals(r_hash, root_hash) then
          return {error = "Not a CatCoin block!"}
        end
        if r_transaction.type ~= "send" then
          return {error = "Not a Send transaction!"}
        end
        if r_transaction.dest ~= state.public_key then
          return {error = "Transaction not for us!"}
        end
        print("CatCoin Received ", state.balance .. " + " .. r_transaction.amount .. " = " .. state.balance + r_transaction.amount)
        return {
          state = update({public_key = state.public_key, balance = state.balance + r_transaction.amount}, this_hash),
          transaction = marker(msg),
        }
      end
      if msg.type == "send" then
        if not msg.signature then
          return {error = {message = "Message not signed!"}}
        end
        if not state.public_key:verify(msg.signature, {
          dest = msg.dest,
          amount = msg.amount,
        parent_hash = parent_hash}) then
        return {error = {message = "Signature does not match!"}}
      end
      if state.balance < msg.amount then
        return {error = "Not enough funds!"}
      end
      print("CatCoin Sent ", state.balance .. " - " .. msg.amount .. " = " .. state.balance - msg.amount)
      return {
        state = update({public_key = state.public_key, balance = state.balance - msg.amount}, this_hash),
        transaction = marker(msg),
      }
    end
  end
end
return function(public_key)
  local this_hash = IO.hash()
  return {state = update({public_key = public_key, balance = 100}, this_hash), transaction = {type = "create"}}
end
end)
