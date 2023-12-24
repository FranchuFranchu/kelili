return kelili.io_run_fun(function()
  -- Get information abou the root block
  -- This allows us to identify data that has been
  -- created us, and prevent other smart contracts
  -- from creating CatCoin transactions (which
  -- would allow them to mint arbitrary amounts of
  -- CatCoin by creating Send transactions)
  local root_hash = IO.hash()
  local marker = IO.mark()
  -- This function builds this contract's update function,
  -- which takes in a transaction and returns
  -- the new updte function and the marked transaction (if it was successful)
  function mk_update(state, parent_hash)
    return function(transaction)
      print("CatCoin transaction ", require("lua/serpent").line(transaction))
      local this_hash = IO.hash()
      if transaction.type == "receive" then
        -- Get the Send transaction that's linked to this
        -- Receive transaction. We must make sure
        -- that it's a legitimate Send transaciton first.
        local remote = IO.call_bubble(transaction.from)
        -- The transaction must be marked.
        local r_transaction, r_hash = IO.open(remote.transaction)
        if not kelili.equals(r_hash, root_hash) then
          return {error = "Not a CatCoin block!"}
        end
        if r_transaction.type ~= "send" then
          return {error = "Not a Send transaction!"}
        end
        if r_transaction.dest ~= parent_hash then
          return {error = "Transaction not for us!"}
        end
        local a = state.balance + r_transaction.amount
        print("CatCoin received ", state.balance .. " + " .. r_transaction.amount .. " = " .. state.balance + r_transaction.amount)
        state.balance = state.balance + r_transaction.amount
        transaction.amount = r_transaction.amount
        return {
          update = mk_update(state, this_hash),
          transaction = marker(transaction),
        }
      end
      if transaction.type == "send" then
        local allow, transaction = state.allow_send(transaction, parent_hash)
        if not allow then
          return {error = {message = "Send message was not allowed", extra = transaction}}
        end
        if state.balance < crypto.U256.from(transaction.amount) then
          return {error = "Not enough funds!"}
        end
        print("CatCoin sent    ", state.balance .. " - " .. transaction.amount .. " = " .. state.balance - transaction.amount)
        state.balance = state.balance - transaction.amount
        return {
          update = mk_update(state, this_hash),
          transaction = marker(transaction),
        }
      end
    end
  end
  return function(transaction)
    if transaction.type ~= "create" then
      return {error = "Not a Create transaction!"}
    end
    local this_hash = IO.hash()
    return {update = mk_update({allow_send = transaction.allow_send, balance = crypto.U256.from(100)}, this_hash), transaction = {type = "create"}}
  end
end)
