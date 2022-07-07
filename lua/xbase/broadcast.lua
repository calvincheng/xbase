local logger = require "xbase.log"
local notify = require "xbase.notify"

local MessageType = {
  Log = "Log",
  Notify = "Notify",
  Execute = "Execute",
}

local Task = {
  OpenLogger = "OpenLogger",
  ReloadLspServer = "ReloadLspServer",
  UpdateStatusline = "UpdateStatusline",
}

local get_messages = function(chunk)
  local chunk = vim.trim(chunk)
  if chunk:find "\n" == nil then
    return { vim.json.decode(chunk) }
  end

  local messages = {}
  for _, value in ipairs(vim.split(chunk, "\n")) do
    messages[#messages + 1] = vim.json.decode(value)
  end
  return messages
end

return function(err, chunk)
  assert(not err, err)
  if chunk == nil or #chunk == 0 then
    return
  end

  vim.schedule(function()
    local messages = get_messages(chunk)

    for _, item in ipairs(messages) do
      local type, msg, level = item.type, item.msg, item.level
      if MessageType.Log == type then
        return logger.log(msg, level)
      end

      if MessageType.Notify == type then
        return notify(msg, level)
      end

      if MessageType.Execute == type then
        local task = msg.task
        if Task.UpdateStatusline == task then
          vim.g.xbase_watch_build_status = msg.value
        elseif Task.OpenLogger then
          logger.toggle(nil, false)
        elseif Task.ReloadLspServer then
          vim.cmd "LspRestart"
        end
      end
    end
  end)
end