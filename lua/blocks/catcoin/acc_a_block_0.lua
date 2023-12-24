local parent_hash = param.cc_hash
local pk = param.pk
return kelili.io_run_fun(function()
  return IO.call(parent_hash)({ type = "create", allow_send = function(transaction, parent_hash)
    if not transaction.signature then
      return false, "Transaction not signed!"
    end
    if parent_hash ~= transaction.parent_hash then
      return false, "Transaction was signed for a different account"
    end
    if not pk:verify(transaction.signature, {
      amount = transaction.amount,
      parent_hash = transaction.parent_hash,
      dest = transaction.dest,
    }) then
      return false, "Signature is wrong!"
    end

    return true, transaction
  end})
end)

