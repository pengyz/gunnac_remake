-- Gun-Nac auto capture for Mesen 2.1.1 (loads as positional CLI arg)
local shotsDir = "/home/peng/workspace/gunnac_remake/shots/"
local frame = 0
local dumped = false

local function log(s)
  local f = io.open("/tmp/mesen_probe/capture.log", "a")
  if f then f:write(s .. "\n") f:close() end
end

local function saveShot(tag)
  local ok, png = pcall(emu.takeScreenshot)
  if ok and png then
    local f = io.open(string.format("%s%s_f%05d.png", shotsDir, tag, frame), "wb")
    if f then f:write(png) f:close(); log("saved shot at frame " .. frame) end
  else
    log("takeScreenshot failed at frame " .. frame)
  end
end

emu.addEventCallback(function(cpuType)
  frame = frame + 1
  if not dumped then
    dumped = true
    local ok, t = pcall(emu.getInput, 0)
    if ok and t then
      local keys = ""
      for k, v in pairs(t) do keys = keys .. k .. "=" .. tostring(v) .. " " end
      log("input keys: " .. keys)
    end
  end
  local function win(a, b) return frame >= a and frame <= b end
  local btn = {}
  if win(240,246) or win(360,366) or win(600,606) or win(900,906) or win(1200,1206) then
    btn.start = true
  end
  if win(300,306) or win(420,426) or win(660,666) or win(1020,1026) then
    btn.a = true
  end
  pcall(function() emu.setInput(btn, 0) end)
end, "inputPolled")

emu.addEventCallback(function(cpuType)
  if frame > 0 and frame % 180 == 0 then saveShot("auto") end
  if frame >= 5400 then
    saveShot("final")
    pcall(emu.stop, 0)
  end
end, "endFrame")

log("script started")
